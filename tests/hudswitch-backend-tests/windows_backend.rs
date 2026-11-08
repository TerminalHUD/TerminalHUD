#![cfg(windows)]

use hudswitch_core::{Availability, Backend, Controller, Event, Presentation, State, VisibleState};
use hudswitch_windows::{Error, LaunchedWindow, WindowsBackend};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::DataExchange::GlobalFindAtomW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

mod cases;
mod fixture;
use fixture::{Fixture, center, click_owned, layered_attributes, wait};
