use crate::{
    app_state::AppState,
    entity::{get_entity_from_value, Entity},
    error::{Error, Result},
    sources::source::Source,
};
use async_trait::async_trait;
use kuma_client::util::ResultLogger;
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashSet, sync::Arc};

#[derive(Debug, Deserialize)]
struct CaddyConfig {
    apps: Option<CaddyApps>,
}

#[derive(Debug, Deserialize)]
struct CaddyApps {
    http: Option<CaddyHttp>,
}

#[derive(Debug, Deserialize)]
struct CaddyHttp {
    servers: Option<std::collections::HashMap<String, CaddyServer>>,
}

#[derive(Debug, Deserialize)]
struct CaddyServer {
    routes: Option<Vec<CaddyRoute>>,
}

#[derive(Debug, Deserialize)]
struct CaddyRoute {
    #[serde(rename = "match")]
    matchers: Option<Vec<CaddyMatcher>>,
}

#[derive(Debug, Deserialize)]
struct CaddyMatcher {
    host: Option<Vec<String>>,
}

async fn fetch_caddy_config(url: &str) -> Result<CaddyConfig> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| Error::IO(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| Error::IO(format!("Failed to fetch Caddy config: {}", e)))?;

    if !response.status().is_success() {
        return Err(Error::IO(format!(
            "Caddy API returned error status: {}",
            response.status()
        )));
    }

    let config: CaddyConfig = response
        .json()
        .await
        .map_err(|e| Error::DeserializeError(format!("Failed to parse Caddy config: {}", e)))?;

    Ok(config)
}

fn extract_hosts(config: &CaddyConfig) -> Vec<String> {
    let mut hosts = HashSet::new();

    if let Some(apps) = &config.apps {
        if let Some(http) = &apps.http {
            if let Some(servers) = &http.servers {
                for server in servers.values() {
                    if let Some(routes) = &server.routes {
                        for route in routes {
                            if let Some(matchers) = &route.matchers {
                                for matcher in matchers {
                                    if let Some(host_list) = &matcher.host {
                                        for host in host_list {
                                            // Remove wildcards and clean up host names
                                            // Caddy uses *.domain.com format for wildcard hosts
                                            let clean_host = host.trim_start_matches("*.").to_string();
                                            if !clean_host.is_empty() {
                                                hosts.insert(clean_host);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut result: Vec<String> = hosts.into_iter().collect();
    result.sort();
    result
}

pub struct CaddySource {
    state: Arc<AppState>,
}

#[async_trait]
impl Source for CaddySource {
    fn name(&self) -> &'static str {
        "Caddy"
    }

    async fn init(&mut self) -> Result<()> {
        log::info!("Initializing Caddy source with URL: {}", self.state.config.caddy.url);
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    async fn get_entities(&mut self) -> Result<Vec<(String, Entity)>> {
        if !self.state.config.caddy.enabled {
            return Ok(vec![]);
        }

        log::debug!("Fetching Caddy config from {}", self.state.config.caddy.url);

        let config = fetch_caddy_config(&self.state.config.caddy.url)
            .await
            .log_warn(std::module_path!(), |e| {
                format!("Failed to fetch Caddy config: {}", e)
            })?;

        let hosts = extract_hosts(&config);
        log::info!("Found {} hosts in Caddy config", hosts.len());

        let mut entities = vec![];

        for host in hosts {
            let protocol = if self.state.config.caddy.use_https {
                "https"
            } else {
                "http"
            };
            let url = format!("{}://{}", protocol, host);

            let monitor_name = match &self.state.config.caddy.monitor_name_prefix {
                Some(prefix) => format!("{}{}", prefix, host),
                None => host.clone(),
            };

            // Generate a unique ID - use the host as-is since AutoKuma IDs support dots
            let id = format!("caddy/{}", host);

            let value = json!({
                "type": "http",
                "name": monitor_name,
                "url": url,
                "interval": 60,
                "retryInterval": 60,
                "maxretries": 3,
            });

            let context = tera::Context::from_value(json!({
                "host": host,
                "url": url,
            }))
            .unwrap();

            match get_entity_from_value(self.state.clone(), id.clone(), value, context) {
                Ok(entity) => {
                    entities.push((id, entity));
                }
                Err(e) => {
                    log::warn!("Failed to create entity for host {}: {}", host, e);
                }
            }
        }

        Ok(entities)
    }
}

impl CaddySource {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hosts() {
        let config_json = r#"{
            "apps": {
                "http": {
                    "servers": {
                        "srv0": {
                            "routes": [
                                {
                                    "match": [
                                        {
                                            "host": ["example.com", "www.example.com"]
                                        }
                                    ]
                                },
                                {
                                    "match": [
                                        {
                                            "host": ["*.wildcard.com"]
                                        }
                                    ]
                                }
                            ]
                        }
                    }
                }
            }
        }"#;

        let config: CaddyConfig = serde_json::from_str(config_json).unwrap();
        let hosts = extract_hosts(&config);

        assert_eq!(hosts.len(), 3);
        assert!(hosts.contains(&"example.com".to_string()));
        assert!(hosts.contains(&"www.example.com".to_string()));
        assert!(hosts.contains(&"wildcard.com".to_string()));
    }

    #[test]
    fn test_extract_hosts_empty_config() {
        let config = CaddyConfig { apps: None };
        let hosts = extract_hosts(&config);
        assert_eq!(hosts.len(), 0);
    }
}
