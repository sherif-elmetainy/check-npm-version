use std::collections::HashMap;
use std::fs::File;
use std::{env, fs, io};
use std::io::BufRead;
use std::path::{Path, PathBuf};
use crate::{log_debug, log_error, log_trace, log_warn};

pub struct NpmConfig {
    registries: HashMap<String, String>,
    auth_tokens: HashMap<String, String>,
}

/// Finds the path to the npm executable by searching in the PATH environment variable.
fn find_npm_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    log_trace!("searching for npm in PATH");

    // Get the PATH environment variable
    let path_var = env::var("PATH").unwrap_or_else(|_| String::new());
    log_trace!("PATH: {}", path_var);

    // Different path separators for different platforms
    let path_separator = if cfg!(windows) { ";" } else { ":" };
    log_trace!("path separator: {}", path_separator);

    // The executable name depends on the platform
    let npm_exe = if cfg!(windows) { "npm.cmd" } else { "npm" };
    log_trace!("npm executable: {}", npm_exe);

    // Iterate over each directory in the PATH
    for dir in path_var.split(path_separator) {
        let path = Path::new(dir).join(npm_exe);
        log_trace!("looking in path: {}", path.to_str().unwrap());
        if path.exists() {
            log_trace!("path exists {}", path.to_str().unwrap());
            return Ok(dir.into());
        } else {
            log_trace!("path does not exist {}", path.to_str().unwrap());
        }
    }

    log_warn!("Could not find npm executable in PATH. Make sure npm is installed and in your PATH.");
    // If npm couldn't be found, return an error
    Err("Could not find npm executable in PATH. Make sure npm is installed and in your PATH.".into())
}


impl NpmConfig {
    pub fn new() -> Self {
        let mut registries = HashMap::new();
        let auth_tokens = HashMap::new();
        registries.insert(String::from(""), String::from("https://registry.npmjs.org/"));
        Self {
            registries,
            auth_tokens,
        }
    }

    pub fn load_for_project(&mut self, project_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // load built in npmrc
        // find where is npm
        let npm_path = find_npm_path();
        if let Ok(npm_path) = npm_path  {
            let builtin_rc = npm_path.join("npmrc");
            log_trace!("looking for builtin npmrc: {}", builtin_rc.to_str().unwrap());
            self.load(builtin_rc.to_str().unwrap())?;
        };

        let home_dir = env::var("HOME");
        if let Ok(home_dir) = home_dir {
            log_trace!("looking for .npmrc in home dir: {}", home_dir);
            let rc_path = Path::new(&home_dir).join(".npmrc");
            self.load(rc_path.to_str().unwrap())?;
        } else {
            log_warn!("HOME environment variable not set");
        }


        let md = fs::metadata(project_path)?;
        if !md.is_dir() {
            let project_path = Path::parent(Path::new(project_path));
            if let Some(project_path) = project_path {
                log_trace!("looking for .npmrc in parent dir: {}", project_path.to_str().unwrap());
                let rc_path = project_path.join(".npmrc");
                log_trace!("looking for .npmrc in parent dir: {}", rc_path.to_str().unwrap());
                self.load(rc_path.to_str().unwrap())?;
            } else {
                log_error!("could not fine parent of project path");
            }
        } else {
            let rc_path = Path::new(project_path).join(".npmrc");
            log_trace!("looking for .npmrc in project dir: {}", rc_path.to_str().unwrap());
            self.load(rc_path.to_str().unwrap())?;
        }
        log_trace!("registries: {:?}", self.registries);

        Ok(())
    }

    fn load(&mut self, rc_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let rc = match fs::exists(rc_path) {
            Ok(false) => {
                log_debug!("{} file not found", rc_path);
                return Ok(());
            }
            Ok(true) => {
                log_debug!("{} file found", rc_path);
                File::open(rc_path)?
            }
            Err(e) => {
                log_warn!("Error checking {} file: {}", rc_path, e);
                return Ok(());
            },
        };
        let mut registries = HashMap::new();
        let mut auth_tokens = HashMap::new();

        let reader = io::BufReader::new(rc);

        // Method 1: Using lines() iterator
        for line in reader.lines() {
            let line = line?;
            log_trace!("Line: {}", line);
            let line = line.trim();
            if line.is_empty() || line.starts_with(";") || line.starts_with("#") {
                continue;
            }

            let mut parts = line.splitn(2, '=');
            let key = parts.next();
            if key.is_none() {
                continue;
            }
            let key = key.unwrap().trim();
            let value = parts.next();
            if value.is_none() {
                continue;
            }
            let value = value.unwrap().trim();
            if key == "registry" {
                registries.insert(String::from(""), String::from(value));
                log_debug!("default registry: {}", value);
                continue;
            }
            if key.starts_with("@") && key.ends_with(":registry") {
                let scope = key.split(':').next().unwrap();
                registries.insert(String::from(scope), String::from(value));
                log_debug!("registry for scope {}: {}", scope, value);
                continue;
            }
            
            if key == "_authToken" {
                auth_tokens.insert(String::from(""), String::from(value));
                log_trace!("default auth token: {}", &value[0..8]);
                continue;
            }
            if key.starts_with("//") && key.ends_with(":_authToken") {
                let url = key.split(':').next().unwrap();
                log_trace!("auth token for url {}: {}", url, &value[0..8]);
                auth_tokens.insert(format!("https:{url}"), String::from(value));
                auth_tokens.insert(format!("http:{url}"), String::from(value));
                continue;           
            }
        }
        

        if auth_tokens.contains_key("") {
            let registry = registries.get("").unwrap();
            let value = auth_tokens.get("").unwrap();
            let value = value.clone();
            auth_tokens.insert(format!("https:{registry}"), value.to_string());
            auth_tokens.insert(format!("http:{registry}"), value.to_string());
            auth_tokens.remove("");
        }


        self.registries.extend(registries);
        self.auth_tokens.extend(auth_tokens);
        Ok(())
    }
    
    pub fn get_scope_registry(&self, scope: &str) -> &String {
        self.registries.get(scope).unwrap_or(self.registries.get("").unwrap())
    }
    pub fn get_auth_token(&self, url: &str) -> Option<&String> {
        self.auth_tokens.get(url)
    }
    pub fn get_package_registry(&self, package: &str) -> &String {
        if package.contains('/') && package.starts_with('@') {
            let scope = package.split('/').next().unwrap();
            return self.get_scope_registry(scope);
        }
        self.registries.get("").unwrap()
    }
}