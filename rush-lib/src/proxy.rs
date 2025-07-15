pub struct Proxy {
    pub host_url: String,
    pub http_proxy_port: u16,
    pub socks_proxy_port: u16,
    pub http_proxy: String,
    pub https_proxy: String,
    pub all_proxy: String,
}

impl Proxy {
    pub fn new(host_url: String, http_proxy_port: u16, socks_proxy_port: u16) -> Self {
        let http_proxy = format!("http://{host_url}:{http_proxy_port}");
        let https_proxy = format!("http://{host_url}:{http_proxy_port}");
        let all_proxy = format!("socks5://{host_url}:{socks_proxy_port}");

        Self {
            host_url,
            http_proxy_port,
            socks_proxy_port,
            http_proxy,
            https_proxy,
            all_proxy,
        }
    }
}
