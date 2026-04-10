use crate::cli::executor::CliExecutor;
use crate::cli::types::DockerPsEntry;
use crate::mdns::config::{ContainerMdnsOverride, MdnsConfig};
use crate::mdns::manager::MdnsManagerInner;
use serde::Serialize;
use std::collections::HashMap;

const DOCKER: &str = "docker";

#[derive(Debug, Serialize, Clone)]
pub struct MdnsSyncResult {
    pub services: Vec<MdnsServiceEntry>,
    pub daemon_running: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct MdnsServiceEntry {
    pub container_id: String,
    pub container_name: String,
    pub hostname: String,
    pub service_type: String,
    pub port: u16,
    pub registered: bool,
    pub auto_registered: bool,
}

pub async fn sync_containers(
    manager: &mut MdnsManagerInner,
    config: &MdnsConfig,
) -> Result<MdnsSyncResult, String> {
    if !manager.is_enabled() {
        return Ok(MdnsSyncResult {
            services: vec![],
            daemon_running: false,
        });
    }

    // 1. 현재 실행 중인 컨테이너 목록 조회
    let entries: Vec<DockerPsEntry> = match CliExecutor::run_json_lines(
        DOCKER,
        &["ps", "--format", "{{json .}}"],
    )
    .await
    {
        Ok(e) => e,
        Err(_) => {
            // Docker/Colima가 실행 중이 아니면 빈 결과 반환
            return Ok(MdnsSyncResult {
                services: manager
                    .list_services()
                    .iter()
                    .map(|s| MdnsServiceEntry {
                        container_id: String::new(),
                        container_name: s.container_name.clone(),
                        hostname: s.hostname.clone(),
                        service_type: s.service_type.clone(),
                        port: s.port,
                        registered: true,
                        auto_registered: s.auto_registered,
                    })
                    .collect(),
                daemon_running: true,
            });
        }
    };

    // 2. 실행 중인 컨테이너를 Map으로 변환
    let running: HashMap<String, &DockerPsEntry> = entries
        .iter()
        .map(|e| (e.names.clone(), e))
        .collect();

    // 3. 종료된 컨테이너의 서비스 해제
    let registered_names: Vec<String> = manager.registered.keys().cloned().collect();
    for name in &registered_names {
        if !running.contains_key(name) {
            let _ = manager.unregister(name);
        }
    }

    // 4. 실행 중인 컨테이너 처리
    let mut result_services = Vec::new();

    for entry in &entries {
        let name = &entry.names;

        // 오버라이드 확인
        if let Some(ovr) = config.container_overrides.get(name) {
            if !ovr.enabled {
                // 비활성화된 오버라이드 → 서비스 해제
                if manager.registered.contains_key(name) {
                    let _ = manager.unregister(name);
                }
                result_services.push(build_entry(entry, ovr, config, false, false));
                continue;
            }

            let hostname = ovr.hostname.as_deref().unwrap_or(name);
            let stype = ovr
                .service_type
                .as_deref()
                .unwrap_or(&config.default_service_type);
            let port = ovr.port.or_else(|| parse_first_host_port(&entry.ports));

            if let Some(port) = port {
                if !manager.registered.contains_key(name) {
                    let _ = manager.register(name, hostname, stype, port, false);
                }
                result_services.push(MdnsServiceEntry {
                    container_id: entry.id.clone(),
                    container_name: name.clone(),
                    hostname: hostname.to_string(),
                    service_type: stype.to_string(),
                    port,
                    registered: manager.registered.contains_key(name),
                    auto_registered: false,
                });
            } else {
                result_services.push(build_entry(entry, ovr, config, false, false));
            }
        } else if config.auto_register {
            // 자동 등록
            if let Some(port) = parse_first_host_port(&entry.ports) {
                if !manager.registered.contains_key(name) {
                    let _ = manager.register(
                        name,
                        name,
                        &config.default_service_type,
                        port,
                        true,
                    );
                }
                result_services.push(MdnsServiceEntry {
                    container_id: entry.id.clone(),
                    container_name: name.clone(),
                    hostname: name.clone(),
                    service_type: config.default_service_type.clone(),
                    port,
                    registered: manager.registered.contains_key(name),
                    auto_registered: true,
                });
            }
            // 포트 미노출 컨테이너는 자동 등록 대상에서 제외
        }
    }

    Ok(MdnsSyncResult {
        services: result_services,
        daemon_running: true,
    })
}

fn build_entry(
    entry: &DockerPsEntry,
    ovr: &ContainerMdnsOverride,
    config: &MdnsConfig,
    registered: bool,
    auto_registered: bool,
) -> MdnsServiceEntry {
    MdnsServiceEntry {
        container_id: entry.id.clone(),
        container_name: entry.names.clone(),
        hostname: ovr
            .hostname
            .clone()
            .unwrap_or_else(|| entry.names.clone()),
        service_type: ovr
            .service_type
            .clone()
            .unwrap_or_else(|| config.default_service_type.clone()),
        port: ovr.port.unwrap_or(0),
        registered,
        auto_registered,
    }
}

pub fn parse_first_host_port(ports: &str) -> Option<u16> {
    // Format: "0.0.0.0:8080->80/tcp, :::8080->80/tcp"
    let segment = ports.split(',').next()?;
    let arrow = segment.find("->")?;
    let before_arrow = &segment[..arrow];
    let colon_pos = before_arrow.rfind(':')?;
    let port_str = &before_arrow[colon_pos + 1..];
    port_str.trim().parse::<u16>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_first_host_port_standard() {
        assert_eq!(parse_first_host_port("0.0.0.0:8080->80/tcp"), Some(8080));
    }

    #[test]
    fn test_parse_first_host_port_multiple() {
        assert_eq!(
            parse_first_host_port("0.0.0.0:8080->80/tcp, :::8080->80/tcp"),
            Some(8080)
        );
    }

    #[test]
    fn test_parse_first_host_port_no_mapping() {
        assert_eq!(parse_first_host_port("80/tcp"), None);
    }

    #[test]
    fn test_parse_first_host_port_empty() {
        assert_eq!(parse_first_host_port(""), None);
    }

    #[test]
    fn test_parse_first_host_port_different_port() {
        assert_eq!(
            parse_first_host_port("0.0.0.0:3000->3000/tcp"),
            Some(3000)
        );
    }
}
