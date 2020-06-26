#[cfg(windows)] extern crate winapi;
use std::io::Error;
use std::mem;
use std::mem::{size_of};
use std::ptr::null_mut;
use winapi::shared::windef::{HICON, HWND};

use winapi::shared::dxgi::*;
use winapi::shared::dxgitype::*;
use winapi::shared::dxgiformat::*;
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::{D3D_DRIVER_TYPE_HARDWARE};

use winapi::um::winuser::{
    MSG,
    DispatchMessageW,
    TranslateMessage,
    GetMessageW,
};

fn win32_string( value : &str ) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new( value ).encode_wide().chain( once( 0 ) ).collect()
}

struct Window {
    handle : HWND,
}

fn handle_message( window : &mut Window ) -> bool {
    unsafe {
        let mut message : MSG = mem::zeroed();
        if GetMessageW( &mut message as *mut MSG, window.handle, 0, 0 ) > 0 {
            TranslateMessage( &message as *const MSG );
            DispatchMessageW( &message as *const MSG );
            true
        } else {
            false
        }
    }
}

fn create_swap_chain(window : HWND){
    unsafe {
        let mut swap_chain_desc : DXGI_SWAP_CHAIN_DESC = mem::zeroed();
        swap_chain_desc.BufferCount = 2;
        swap_chain_desc.Windowed = 1;
        swap_chain_desc.BufferDesc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
        swap_chain_desc.SampleDesc.Count = 1;
        swap_chain_desc.SampleDesc.Quality = 0;
        swap_chain_desc.SwapEffect = DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL;
        swap_chain_desc.OutputWindow = window;

        let mut _swap_chain : *mut IDXGISwapChain = mem::zeroed();
        let mut _device : *mut ID3D11Device = mem::zeroed();
        let mut _device_context : *mut ID3D11DeviceContext = mem::zeroed();

        let res = D3D11CreateDeviceAndSwapChain(
            null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            null_mut(),
            0,
            null_mut(),
            0,
            7, 
            &swap_chain_desc,
            &mut _swap_chain as _,
            &mut _device as _,
            null_mut(),
            &mut _device_context as _
        );
        
        println!("Res: {}", res);

        // let back_buffer : *mut ID3D11Resource = mem::zeroed();
        // let render_target : *mut ID3D11RenderTargetView = mem::zeroed();
    }
}

fn create_window( name: &str, title: &str) -> Result<Window, Error>{
    use winapi::um::winuser::{CW_USEDEFAULT, WS_VISIBLE, WS_OVERLAPPEDWINDOW, CreateWindowExW, DefWindowProcW, WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW, RegisterClassExW};

    //Convert strings to correct format
    let name = win32_string( name );
    let title = win32_string( title );

    unsafe {
        let hinstance = winapi::um::libloaderapi::GetModuleHandleW( null_mut() );   
        let wnd_class = WNDCLASSEXW {
            cbSize : size_of::<WNDCLASSEXW>()as u32,
            hIconSm : 0 as HICON,
            style : CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc : Some( DefWindowProcW ),
            hInstance : hinstance,
            lpszClassName : name.as_ptr(),
            cbClsExtra : 0,
            cbWndExtra : 0,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
        };

        RegisterClassExW( &wnd_class );
        
        let handle = CreateWindowExW(
            0,
            name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut() );

        if handle.is_null() {
            Err( Error::last_os_error() )
        } else {
            create_swap_chain(handle);
            Ok( Window { handle } )
        }
    }
}

fn main() {
    let name = "winclass1";
    let title = "win_title";

    let mut window = create_window(&name, &title).unwrap();

    loop {
        if !handle_message( &mut window ) {
            break;
        }
    }
    
}