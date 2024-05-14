#[cfg(test)]
mod server_test {

    use std::fs;
    use toml;
    #[test]
    fn take_screenshot() {
        println!("")
    }

    #[test]
    fn read_config(){

    }

    /// SESMAN TESTS

    #[cfg(target_os = "linux")]
    #[test]
    fn login(){
        use pam::Client;
        let mut client = Client::with_password("system-auth")
            .expect("Failed to init PAM client.");
        // Preset the login & password we will use for authentication
        client.conversation_mut().set_credentials("login", "password");
        // Actually try to authenticate:
        client.authenticate().expect("Authentication failed!");
        // Now that we are authenticated, it's possible to open a session:
        client.open_session().expect("Failed to open a session!");
    }
}