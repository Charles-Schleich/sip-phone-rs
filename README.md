# sip-phone-rs
A minimal coverage library that utilizes pjproject to achive telephony in rust. 

## Exported Functions
``` rust
// Essentially  init + add_transport + start_telephony wrapped and ordered correctly 
fn initialize_telephony( logLevel:u32
                       , incommingCallBehaviour:OnIncommingCall
                       , port:u32
                       , transportmode :TransportMode)              -> Result<i8,TelephonyError> {}

// Standard init function, log level 0(min)-4(max), two defaults to handle calls
fn init(loglevel : u32 , incommingCallBehaviour: OnIncommingCall )  -> Result<i8,TelephonyError> {}

// Setting Transport mode (UDP/TCP/TLS) and port of Telephony
fn add_transport(port: u32, mode: TransportMode )                   -> Result<i8,TelephonyError> {}

// Standard Account Setup 
fn accountSetup(username : String, uri : String, password : String) -> Result<i8,TelephonyError> {}

// Function to send DTMF tones. (Single or multiple digits)
fn send_dtmfndDTMF(digit:u32)                                       -> Result<i8,TelephonyError> {}

// Starts_telephony service 
fn start_telephony()                                                -> Result<i8,TelephonyError> {}

// Places a call to the phone number, given a domain (same as regstration_domain) 
fn make_call(phoneNumber: &str, domain : &str )                     -> Result<i8,TelephonyError> {}

// Destroys every telephone. Ever.
fn destroy_telephony()                                          -> Result<i8,TelephonyError> {}

// Hangs up calls
fn hangup_call()                                                    -> () {}
```

## Exported Eums
``` rust
#[derive(Error, Debug)]
pub  enum  TelephonyError {
    #[error("Cannot create a Telephony instance")]
    CreationError(String),
    #[error("Internal Config invalid")]
    ConfigError(String),
    #[error("Cannot Initialize Telephony instance")]
    InitializationError(String),
    #[error("Cannot Initialize Telephony-Transport")]
    TransportError(String),
    #[error("Cannot Send DTMF Tone")]
    DTMFError(String),
    #[error("Could not Create Call")]
    CallCreationError(String),
    #[error("Account Creation Error")]
    AccountCreationError(String),
    #[error("Telephony Start Error")]
    TelephonyStartError(String),
    #[error("Telephony destruction Error")]
    TelephonyDestroyError(String),
    #[error("Input Error")]
    InputValueError(String),
}

#[derive(Debug)]
pub  enum  TransportMode {
    TCP,
    UDP,
    TLS,
    UDP6,
    TCP6,
    TLS6
}

#[derive(Debug)]
pub enum OnIncommingCall {
    AutoAnswer,
    Ignore
}
```

## Example
Cargo.toml file
``` toml
[dependencies]
te-telephony-rs = { git = "https://github.com/Charles-Schleich/sip-phone-rs.git", branch = "master" }
```
There may be a step here that involves using git credential store helper on your machine, for Toml uses that for non-crates.io sources of crates.


main.rs file
``` rust 
extern crate telephony as tel;

pub fn main() {
    //Setup
    let init_success = tel::initialize_telephony(0,tel::OnIncommingCall::AutoAnswer,5060,tel::TransportMode::UDP);

    //Account Setup
    let username = String::from ("USERNAME");
    let reg_uri = String::from ("ADDRESS");
    let password = String::from ("PASSWORD");
    tel::accountSetup(username, reg_uri, password);

    loop {
        use std::io;
        let mut input = String::new();
        println!("-------------------------------------");
        println!("Type: 'h' to hangup all calls,\n    : 'd' to send 123456 DTMF tone\n    : 'f' to send 2 DTMF tone\n    : 'c' to call\n    : 'q' to quit");
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if input == "q\n" { break; }
                if input == "h\n" { unsafe{ tel::hangup_calls()}; }
                if input == "d\n" { tel::send_dtmf(123456); }
                if input == "f\n" { tel::send_dtmf(2); }
                if input == "c\n" { 
                    let mut telNum = String::new();
                    println!("Type In telephone number");
                    
                    match io::stdin().read_line(&mut telNum) {
                        Ok(n) => {
                            let len = telNum.len();
                            telNum.truncate(len - 1);
                            tel::make_call(&telNum,"DOMAIN"); }
                        Err(error) => println!("error: {}", error),
                    }
                } 
            }
            Err(error) => println!("error: {}", error),
        }
    }

    // Destroy and cleanup telephony
    tel::destroy_telephony();
}
```
