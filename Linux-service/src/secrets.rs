use keyring::Entry;

fn entry_for(provider: &str) -> Result<Entry, keyring::Error> {
    // Service name identifies our app; account is the provider name
    Entry::new("voxora-service", provider)
}

pub fn save_key(provider: &str, api_key: &str) -> Result<(), keyring::Error> {
    entry_for(provider)?.set_password(api_key)
}

pub fn delete_key(provider: &str) -> Result<(), keyring::Error> {
    // Some platforms return an error if not present; ignore that gracefully
    match entry_for(provider)?.delete_password() {
        Ok(()) => Ok(()),
        Err(e) => {
            // If no entry, treat as success
            if matches!(e, keyring::Error::NoEntry) { Ok(()) } else { Err(e) }
        }
    }
}

pub fn has_key(provider: &str) -> bool {
    entry_for(provider).and_then(|e| e.get_password()).map(|_| true).unwrap_or(false)
}

pub fn get_key(provider: &str) -> Option<String> {
    entry_for(provider).ok().and_then(|e| e.get_password().ok())
}
