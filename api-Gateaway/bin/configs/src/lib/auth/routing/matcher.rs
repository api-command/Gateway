use regex::Regex;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub methods: Vec<String>,
    pub backend: String,
    pub rewrite: Option<RewriteRule>,
    pub prefix: bool,
    pub regex: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RewriteRule {
    pub from: Regex,
    pub to: String,
}

#[derive(Debug, Error)]
pub enum MatchError {
    #[error("No route found")]
    RouteNotFound,
    #[error("Method not allowed")]
    MethodNotAllowed,
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(#[from] regex::Error),
}

pub struct RouteMatcher {
    routes: Vec<Route>,
    regex_cache: HashMap<String, Regex>,
}

impl RouteMatcher {
    pub fn new(routes: Vec<Route>) -> Result<Self, MatchError> {
        let mut regex_cache = HashMap::new();
        
        for route in &routes {
            if let Some(pattern) = &route.regex {
                regex_cache.insert(pattern.clone(), Regex::new(pattern)?);
            }
        }
        
        Ok(Self { routes, regex_cache })
    }

    pub fn find_route(
        &self,
        path: &str,
        method: &str
    ) -> Result<(Route, HashMap<String, String>), MatchError> {
        for route in &self.routes {
            // Method check
            if !route.methods.iter().any(|m| m.eq_ignore_ascii_case(method)) {
                continue;
            }

            // Exact match
            if path == route.path {
                return Ok((route.clone(), HashMap::new()));
            }

            // Prefix match
            if route.prefix && path.starts_with(&route.path) {
                return Ok((route.clone(), self.extract_prefix_params(route, path)));
            }

            // Regex match
            if let Some(pattern) = &route.regex {
                if let Some(captures) = self.regex_cache[pattern].captures(path) {
                    return Ok((route.clone(), self.extract_regex_params(pattern, captures)));
                }
            }
        }
        
        Err(MatchError::RouteNotFound)
    }

    fn extract_prefix_params(&self, route: &Route, path: &str) -> HashMap<String, String> {
        let remainder = path.replacen(&route.path, "", 1);
        let mut params = HashMap::new();
        params.insert("prefix_remainder".to_string(), remainder);
        params
    }

    fn extract_regex_params(
        &self,
        pattern: &str,
        captures: regex::Captures
    ) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let re = &self.regex_cache[pattern];
        
        for name in re.capture_names().flatten() {
            if let Some(value) = captures.name(name) {
                params.insert(name.to_string(), value.as_str().to_string());
            }
        }
        
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_route() -> Route {
        Route {
            path: "/users".to_string(),
            methods: vec!["GET".to_string()],
            backend: "http://user-service".to_string(),
            rewrite: None,
            prefix: false,
            regex: None,
        }
    }

    #[test]
    fn test_exact_match() {
        let matcher = RouteMatcher::new(vec![test_route()]).unwrap();
        let (route, params) = matcher.find_route("/users", "GET").unwrap();
        assert_eq!(route.backend, "http://user-service");
        assert!(params.is_empty());
    }
}