#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::env;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));