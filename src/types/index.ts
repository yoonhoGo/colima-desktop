export interface Container {
  id: string;
  name: string;
  image: string;
  state: string;
  status: string;
  ports: string;
  created_at: string;
  compose_project: string | null;
  compose_service: string | null;
}

export interface Image {
  id: string;
  repository: string;
  tag: string;
  size: string;
  created_at: string;
  in_use: boolean;
}

export interface ColimaStatus {
  running: boolean;
  runtime: string;
  arch: string;
  cpus: number;
  memory_gib: number;
  disk_gib: number;
}

export interface VmSettings {
  cpus: number;
  memory_gib: number;
  disk_gib: number;
  runtime: string;
  network_address: string;
}

export interface HostInfo {
  cpus: number;
  memory_gib: number;
}

export interface Volume {
  name: string;
  driver: string;
  scope: string;
  mountpoint: string;
  labels: string;
  size: string;
}

export interface Network {
  id: string;
  name: string;
  driver: string;
  scope: string;
  ipv6: boolean;
  internal: boolean;
  labels: string;
}

export interface MountEntry {
  location: string;
  writable: boolean;
}

export interface MountSettings {
  mounts: MountEntry[];
  mount_type: string;
  mount_inotify: boolean;
}

export interface DnsHostEntry {
  hostname: string;
  ip: string;
}

export interface NetworkSettings {
  dns: string[];
  dns_hosts: DnsHostEntry[];
  network_address: boolean;
  network_mode: string;
  gateway_address: string;
  network_interface: string;
  port_forwarder: string;
}

export interface DockerDaemonSettings {
  insecure_registries: string[];
  registry_mirrors: string[];
}
