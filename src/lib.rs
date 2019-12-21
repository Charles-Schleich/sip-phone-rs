#![allow(non_snake_case)]
#![warn(dead_code)]
// #![warn(unused_parens)] 
extern crate pjproject as pj;

use thiserror::Error;
use std::ffi::CString;
use std::os::raw::c_char;
use std::mem::MaybeUninit;
use std::convert::TryInto;

// GLOBAL VARS
static REALM_GLOBAL: &'static str = "asterisk";

pub fn initialize_telephony(logLevel:u32, incommingCallBehaviour:OnIncommingCall, port:u32, transportmode :TransportMode) -> Result<i8,TelephonyError> {

    // INIT
    let initResult = init(logLevel,incommingCallBehaviour);
    match(initResult){
        Ok(_) => (),
        Err(x) => return Err(x),
    };

    // ADD TRANSPORT
    let transportResult = add_transport(port,transportmode);
    match(transportResult){
        Ok(_) => (),
        Err(x) => return Err(x),
    };

    // START
    let startResult = start_telephony();
    match(startResult){
        Ok(_) => (),
        Err(x) => return Err(x),
    };
    Ok(0)
}


//   ______  _   _  _    _  __  __   _____ 
//  |  ____|| \ | || |  | ||  \/  | / ____|
//  | |__   |  \| || |  | || \  / || (___  
//  |  __|  | . ` || |  | || |\/| | \___ \ 
//  | |____ | |\  || |__| || |  | | ____) |
//  |______||_| \_| \____/ |_|  |_||_____/ 
                                 
#[derive(Error, Debug)]
pub enum TelephonyError {
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
pub enum TransportMode {
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

//    _____        _                   ______                    _    _                    
//   / ____|      | |                 |  ____|                  | |  (_)                   
//  | (___    ___ | |_  _   _  _ __   | |__  _   _  _ __    ___ | |_  _   ___   _ __   ___ 
//   \___ \  / _ \| __|| | | || '_ \  |  __|| | | || '_ \  / __|| __|| | / _ \ | '_ \ / __|
//   ____) ||  __/| |_ | |_| || |_) | | |   | |_| || | | || (__ | |_ | || (_) || | | |\__ \
//  |_____/  \___| \__| \__,_|| .__/  |_|    \__,_||_| |_| \___| \__||_| \___/ |_| |_||___/
//                            | |                                                          
//                            |_|                                                          

pub fn init(loglevel : u32 , incomming_call_behaviour: OnIncommingCall ) -> Result<i8,TelephonyError> { 
        
    let status : pj::pj_status_t;

    status = unsafe{pj::pjsua_create()};

    if (status != pj::pj_constants__PJ_SUCCESS as i32){ 
        println!("Error in pjsua_create, status:= {}",status);
        error_exit("Error in pjsua_create");
        return Err(TelephonyError::CreationError("Could not Create Telephony Instance".to_string()));
    }

    let status : pj::pj_status_t;    
    let mut cfg =  unsafe {
        let mut cfg: MaybeUninit<pj::pjsua_config> = MaybeUninit::uninit();
        pj::pjsua_config_default(cfg.as_mut_ptr());
        cfg.assume_init()
    };

    // cfg.cb.on_incoming_call = Some(aux_on_incomming_call());
    match(incomming_call_behaviour){
        OnIncommingCall::AutoAnswer => cfg.cb.on_incoming_call = Some(on_incoming_call),
        OnIncommingCall::Ignore => cfg.cb.on_incoming_call = Some(on_incoming_call_ignore),
    }

    cfg.cb.on_call_media_state = Some(on_call_media_state);
    cfg.cb.on_call_state = Some(on_call_state);

	let mut log_cfg =  unsafe {
        let mut log_cfg: MaybeUninit<pj::pjsua_logging_config> = MaybeUninit::uninit();
        pj::pjsua_logging_config_default(log_cfg.as_mut_ptr());
        log_cfg.assume_init()
    };
	
    log_cfg.console_level = loglevel;
    
	status = unsafe{pj::pjsua_init(&cfg, &log_cfg, std::ptr::null())};
    if (status != pj::pj_constants__PJ_SUCCESS as i32){ 
        error_exit("Error in pjsua_init");
        println!("Error in pjsua_init, status:= {}",status);
        return Err(TelephonyError::InitializationError("Error in pjsua_init".to_string()));
    }
    return Ok(0);
}

pub fn add_transport(port: u32, mode: TransportMode ) -> Result <i8,TelephonyError>{
     /* Add UDP transport. */
    println!("INIT TRANSPORT CFG");

    let mut transport_cfg =  unsafe {
        let mut transport_cfg: MaybeUninit<pj::pjsua_transport_config> = MaybeUninit::uninit();
        pj::pjsua_transport_config_default(transport_cfg.as_mut_ptr());
        transport_cfg.assume_init()
    };    

    transport_cfg.port = port;
    let transportMode = match(mode){
        TransportMode::TCP  => pj::pjsip_transport_type_e_PJSIP_TRANSPORT_TCP,
        TransportMode::TLS  => pj::pjsip_transport_type_e_PJSIP_TRANSPORT_TLS,
        TransportMode::UDP6 => pj::pjsip_transport_type_e_PJSIP_TRANSPORT_UDP6,
        TransportMode::UDP  => pj::pjsip_transport_type_e_PJSIP_TRANSPORT_UDP,
        TransportMode::TCP6 => pj::pjsip_transport_type_e_PJSIP_TRANSPORT_TCP6,
        TransportMode::TLS6 => pj::pjsip_transport_type_e_PJSIP_TRANSPORT_TLS6
    };

    let mut status =  unsafe {
        let mut transport_id: MaybeUninit<pj::pjsua_transport_id> = MaybeUninit::uninit();
        pj::pjsua_transport_create( transportMode, &transport_cfg, transport_id.as_mut_ptr())
    };

    if (status != pj::pj_constants__PJ_SUCCESS as i32 ) {
        return Err(TelephonyError::TransportError("Error of some kind".to_string()));
    }
    return Ok(0);
}

pub fn start_telephony() -> Result <i8,TelephonyError>{
    let status = unsafe{pj::pjsua_start()};
    if (status != pj::pj_constants__PJ_SUCCESS as i32) {
        println!("Error starting pjsua, status = {}",status);
        error_exit("Error starting pjsua");
        return Err(TelephonyError::TelephonyStartError("Could not Start Telephony".to_string()));
        };
    Ok(0)
}

pub fn accountSetup(username : String, uri : String, password : String) -> Result<i8,TelephonyError> {
    println!("ACCOUNT SETUP");
    let status : pj::pj_status_t;
    let mut acc_cfg =  unsafe {
        let mut acc_cfg: Box<MaybeUninit<pj::pjsua_acc_config>> = Box::new(MaybeUninit::uninit());
        pj::pjsua_acc_config_default(acc_cfg.as_mut_ptr());
        acc_cfg
    };
    let acc_cfg_ref = unsafe { &mut *acc_cfg.as_mut_ptr() }; 

    let acc_id : String      = ["sip:".to_string(), username.clone(), "@".to_string(),uri.clone()].concat();
    let reg_uri : String    = ["sip:".to_string(), uri.clone()].concat();
    let realm : String      = REALM_GLOBAL.to_owned();
    // "asterisk".to_owned();
    let scheme : String     = uri;
    let username : String   = username;
    let data : String       = password;

    let acc_id_pj_str_t = match(make_pj_str_t(acc_id)){
        Err(x)=> return Err(x),
        Ok(y)=>y 
    }; 
    let reg_uri_pj_str_t = match(make_pj_str_t(reg_uri)){
        Err(x)=> return Err(x),
        Ok(y)=>y 
    }; 
    let realm_pj_str_t = match(make_pj_str_t(realm)){
        Err(x)=> return Err(x),
        Ok(y)=>y 
    }; 
    let scheme_pj_str_t = match(make_pj_str_t(scheme)){
        Err(x)=> return Err(x),
        Ok(y)=>y 
    }; 
    let username_pj_str_t = match(make_pj_str_t(username)){
        Err(x)=> return Err(x),
        Ok(y)=>y 
    };
    let data_pj_str_t = match(make_pj_str_t(data)){
        Err(x)=> return Err(x),
        Ok(y)=>y 
    };

    // Setting members of the struct
    acc_cfg_ref.id = acc_id_pj_str_t ;
    acc_cfg_ref.reg_uri = reg_uri_pj_str_t;
    acc_cfg_ref.cred_count = 1;
    acc_cfg_ref.cred_info[0].realm = realm_pj_str_t;
    acc_cfg_ref.cred_info[0].scheme = scheme_pj_str_t;
    acc_cfg_ref.cred_info[0].username = username_pj_str_t;
    acc_cfg_ref.cred_info[0].data_type = pj::pjsip_cred_data_type_PJSIP_CRED_DATA_PLAIN_PASSWD.try_into().unwrap();
    acc_cfg_ref.cred_info[0].data = data_pj_str_t;

    let acc_id : pj::pjsua_acc_id;
    acc_id = 0 ;
    status = unsafe {pj::pjsua_acc_add(acc_cfg_ref, pj::pj_constants__PJ_TRUE.try_into().unwrap(), acc_id as *mut i32 )} ;

    // let acc_id_c_string_fromraw = unsafe {CString::from_raw( acc_id_myptr)}; // Might need this at a later stage
    
    println!("Status of pjsua Acc add : {}" , status);
    if status != pj::pj_constants__PJ_SUCCESS as i32  { 
        println!("Error Adding Account, status = {}", status);
        error_exit("Error Adding Account");
        return Err(TelephonyError::AccountCreationError("Error Adding Account".to_string()));
    }
    return Ok(0);
}

//   _    _        _                    
//  | |  | |      | |                   
//  | |__| |  ___ | | _ __    ___  _ __ 
//  |  __  | / _ \| || '_ \  / _ \| '__|
//  | |  | ||  __/| || |_) ||  __/| |   
//  |_|  |_| \___||_|| .__/  \___||_|   
//                   | |                
//                   |_|                
    
pub fn make_pj_str_t_OLD(input : String ) -> pj::pj_str_t {
    // TODO: See about getting Rust Strings to work, 
    //       Currently failing at runtime
    // println!("-----------------------------------");
    let len = input.len() as ::std::os::raw::c_long;
    let bytes = input.clone().into_bytes();
    let mut cchars: Vec<c_char> = bytes.into_iter().map(|b| b as c_char).collect();
    let slice = cchars.as_mut_slice();
    let input_ptr: *mut c_char = slice.as_mut_ptr();
    pj::pj_str_t{ slen: len, ptr: input_ptr}
}

pub fn make_pj_str_t(input : String ) -> Result<pj::pj_str_t,TelephonyError> {
    let len = input.len() as ::std::os::raw::c_long;
    let input_c_string = CString::new(input.clone());
    match(input_c_string){
        Err(_x) => {
            let errMessage : String  = ["Could not use input value: '".to_string(), input, "' Contained Null Byte".to_string() ].concat();
            return Err(TelephonyError::InputValueError(errMessage));
        },
        Ok(c_string) => {
            let input_ptr = c_string.into_raw();
            // If memory leak, This line below may be the fix
            // let _ = unsafe{CString::from_raw(input_ptr)};
            return Ok(  pj::pj_str_t { 
                            slen: len, 
                            ptr: input_ptr
                        }
                    );
        },
    }
}


//    _____        _  _  _                   _         
//   / ____|      | || || |                 | |        
//  | |      __ _ | || || |__    __ _   ___ | | __ ___ 
//  | |     / _` || || || '_ \  / _` | / __|| |/ // __|
//  | |____| (_| || || || |_) || (_| || (__ |   < \__ \
//   \_____|\__,_||_||_||_.__/  \__,_| \___||_|\_\|___/

// fn aux_on_incomming_call() -> extern "C" fn(acc_id: pj::pjsua_acc_id, call_id: pj::pjsua_call_id, rdata: *mut pj::pjsip_rx_data) {
//     on_incoming_call
// }

extern "C" fn on_incoming_call(acc_id: pj::pjsua_acc_id, call_id: pj::pjsua_call_id, rdata: *mut pj::pjsip_rx_data) {
    // let ci = unsafe {
    //     let mut ci : MaybeUninit<pj::pjsua_call_info> = MaybeUninit::uninit();
    //     pj::pjsua_call_get_info(call_id, ci.as_mut_ptr());
    //     ci.assume_init()
    // };
    unsafe{ pj::pjsua_call_answer(call_id, 200, std::ptr::null(), std::ptr::null()); } 
    println!("The call id is: {}", call_id);
}

extern "C" fn on_incoming_call_ignore(acc_id: pj::pjsua_acc_id, call_id: pj::pjsua_call_id, rdata: *mut pj::pjsip_rx_data) {
    // let ci = unsafe {
    //     let mut ci : MaybeUninit<pj::pjsua_call_info> = MaybeUninit::uninit();
    //     pj::pjsua_call_get_info(call_id, ci.as_mut_ptr());
    //     ci.assume_init()
    // };
    println!("The call id is: {}", call_id);
}

extern "C" fn on_call_media_state(call_id: pj::pjsua_call_id) {
    let ci = unsafe {
        let mut ci : MaybeUninit<pj::pjsua_call_info> = MaybeUninit::uninit();
        pj::pjsua_call_get_info(call_id, ci.as_mut_ptr());
        ci.assume_init()
    };
    
    if ci.media_status == pj::pjsua_call_media_status_PJSUA_CALL_MEDIA_ACTIVE {
        unsafe { 
            pj::pjsua_conf_connect(ci.conf_slot, 0);
            pj::pjsua_conf_connect(0,ci.conf_slot);
        }
    }
}

extern "C" fn on_call_state(call_id: pj::pjsua_call_id, e: *mut pj::pjsip_event){
    let ci = unsafe {
        let mut ci : MaybeUninit<pj::pjsua_call_info> = MaybeUninit::uninit();
        pj::pjsua_call_get_info(call_id, ci.as_mut_ptr());
        ci.assume_init()
    };
}


//   _____     _  _           ______                    _    _                    
//  |_   _|   | || |         |  ____|                  | |  (_)                   
//    | |   __| || |  ___    | |__  _   _  _ __    ___ | |_  _   ___   _ __   ___ 
//    | |  / _` || | / _ \   |  __|| | | || '_ \  / __|| __|| | / _ \ | '_ \ / __|
//   _| |_| (_| || ||  __/   | |   | |_| || | | || (__ | |_ | || (_) || | | |\__ \
//  |_____|\__,_||_| \___|   |_|    \__,_||_| |_| \___| \__||_| \___/ |_| |_||___/

pub fn make_call(phone_number: &str, domain : &str ) -> Result<i8,TelephonyError>{

    // TODO: Check Phone number isnt garbage string
    let call_extension : String  = ["sip:".to_string(), phone_number.to_string(), "@".to_string(), domain.to_string()].concat();
    let len = call_extension.len() as ::std::os::raw::c_long;
    let call_extension_c_string = CString::new(call_extension);
    let call_extension_c_string_ok = match(call_extension_c_string) {
        Err(_y) => return (Err(TelephonyError::CallCreationError("Phone number or Domain supplied could not be represented as a C-String".to_string()))),
        Ok(x) => x
    };

    let call_extension_myptr = call_extension_c_string_ok.into_raw();
    let call_extension_pj_str_t = pj::pj_str_t { slen: len , ptr: call_extension_myptr };
    let ptr_call_extension_pj_str_t = &call_extension_pj_str_t as *const _;

    let user_data_null: *mut ::std::os::raw::c_void = &mut 0 as *mut _ as *mut ::std::os::raw::c_void;
    let opt = 0 as *mut pj::pjsua_call_setting;
    let make_call_restult = unsafe {pj::pjsua_call_make_call( 0 , ptr_call_extension_pj_str_t , opt, user_data_null, 0 as *mut  pj::pjsua_msg_data , 0 as *mut pj::pjsua_call_id)};
    if make_call_restult!=0 {
        return Err(TelephonyError::CallCreationError("Could not place Call".to_string()));
    }
    return Ok(0);
}


//   _____                       _  _      ______                    _    _                    
//  |_   _|                     | || |    |  ____|                  | |  (_)                   
//    | |   _ __      ___  __ _ | || |    | |__  _   _  _ __    ___ | |_  _   ___   _ __   ___ 
//    | |  | '_ \    / __|/ _` || || |    |  __|| | | || '_ \  / __|| __|| | / _ \ | '_ \ / __|
//   _| |_ | | | |  | (__| (_| || || |    | |   | |_| || | | || (__ | |_ | || (_) || | | |\__ \
//  |_____||_| |_|   \___|\__,_||_||_|    |_|    \__,_||_| |_| \___| \__||_| \___/ |_| |_||___/

pub fn send_dtmf(digit:u32) -> Result<i8,TelephonyError> {

    let digits : String       = digit.to_string();
    let digits_pj_str_t = match(make_pj_str_t(digits)){
        Err(_e) => return Err(TelephonyError::DTMFError("Cannot Send DTMF Tone, digits contain Null Somehow".to_string())),
        Ok(v) => v
    };

    let dtmf_tones = pj::pjsua_call_send_dtmf_param {
        method   : pj::pjsua_dtmf_method_PJSUA_DTMF_METHOD_RFC2833,
        duration : pj::PJSUA_CALL_SEND_DTMF_DURATION_DEFAULT,
        digits   : digits_pj_str_t
    };
    let status = unsafe{ pj::pjsua_call_send_dtmf(0, &dtmf_tones )};
    if status!=0{
        return Err(TelephonyError::DTMFError("Cannot Send DTMF Tone".to_string()));  
    }
    return Ok(0); 
}

pub fn hangup_calls(){
    unsafe{ pj::pjsua_call_hangup_all()}; 
}

//   _____              _                       _    _                    ______                    _    _                    
//  |  __ \            | |                     | |  (_)                  |  ____|                  | |  (_)                   
//  | |  | |  ___  ___ | |_  _ __  _   _   ___ | |_  _   ___   _ __      | |__  _   _  _ __    ___ | |_  _   ___   _ __   ___ 
//  | |  | | / _ \/ __|| __|| '__|| | | | / __|| __|| | / _ \ | '_ \     |  __|| | | || '_ \  / __|| __|| | / _ \ | '_ \ / __|
//  | |__| ||  __/\__ \| |_ | |   | |_| || (__ | |_ | || (_) || | | |    | |   | |_| || | | || (__ | |_ | || (_) || | | |\__ \
//  |_____/  \___||___/ \__||_|    \__,_| \___| \__||_| \___/ |_| |_|    |_|    \__,_||_| |_| \___| \__||_| \___/ |_| |_||___/

fn error_exit(err_msg: &str ){
    println!("Exiting PJSUA {}",err_msg);
    let err : *const c_char = CString::new("Error Here").expect("CString::new failed").as_ptr();
    let thisfile = CString::new("main.rs").expect("CString::new failed").as_ptr();
    unsafe{pj::pjsua_perror(thisfile, err  , 2)};
    unsafe{pj::pjsua_destroy()};
    unsafe{pj::exit(1)};
}

pub fn destroy_telephony()-> Result<i8,TelephonyError>{
    println!("Destroy telephony");
    let status = unsafe{pj::pjsua_destroy()};
    if status!=0{
        return Err(TelephonyError::TelephonyDestroyError("Error Occured during Telephony Destruction".to_string()));
    }
    Ok(0)
}


// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
