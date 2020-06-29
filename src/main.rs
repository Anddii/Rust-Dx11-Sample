#[cfg(windows)] extern crate winapi;
use std::io::Error;
use std::mem;
use std::mem::{size_of};
use std::ptr::null_mut;

use winapi::shared::minwindef::LPVOID;
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

struct D11Devices{
    _swap_chain: *mut IDXGISwapChain,
    _device: *mut ID3D11Device,
    _device_context: *mut ID3D11DeviceContext,
    _back_buffer : *mut ID3D11Texture2D,
    _render_target : *mut ID3D11RenderTargetView,
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

fn create_swap_chain(window : HWND, devices : &mut D11Devices){

    unsafe {
        let mut swap_chain_desc : DXGI_SWAP_CHAIN_DESC = mem::zeroed();
        swap_chain_desc.BufferCount = 2;
        swap_chain_desc.Windowed = 1;
        swap_chain_desc.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
        swap_chain_desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
        swap_chain_desc.SampleDesc.Count = 1;
        swap_chain_desc.SampleDesc.Quality = 0;
        swap_chain_desc.SwapEffect = DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL;
        swap_chain_desc.OutputWindow = window;

        #[cfg(debug_assertions)]
        let creation_flags = D3D11_CREATE_DEVICE_DEBUG;
        
        #[cfg(not(debug_assertions))]
        let creation_flags = 0;

        let res = D3D11CreateDeviceAndSwapChain(
            null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            null_mut(),
            creation_flags,
            null_mut(),
            0,
            7, 
            &swap_chain_desc,
            &mut devices._swap_chain as _,
            &mut devices._device as _,
            null_mut(),
            &mut devices._device_context as _
        );

        println!("D3D11CreateDeviceAndSwapChain: {}", res);

        if let Some(swap_chain) = devices._swap_chain.as_ref(){
            let res = swap_chain.GetBuffer(0, &IID_ID3D11Texture2D, &mut devices._back_buffer as *mut _ as *mut LPVOID);
            println!("GetBuffer: {}", res);
        }

        if let Some(device) = devices._device.as_ref(){
            let res = device.CreateRenderTargetView(devices._back_buffer as *mut _, null_mut(), &mut devices._render_target as *mut _ as *mut _);
            println!("Create Device: {}", res);
        }

        if let Some(device_context) = devices._device_context.as_ref(){
            device_context.OMSetRenderTargets(1, &mut devices._render_target as _, null_mut());
        }
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
            Ok( Window { handle } )
        }
    }
}

fn main() {
    let name = "winclass1";
    let title = "win_title";

    let mut d11_devices = D11Devices{
        _swap_chain: unsafe{mem::zeroed()},
        _device: unsafe{mem::zeroed()},
        _device_context: unsafe{mem::zeroed()},
        _back_buffer: unsafe{mem::zeroed()},
        _render_target : unsafe{mem::zeroed()},
    };

    let mut window = create_window(&name, &title).unwrap();
    create_swap_chain(window.handle, &mut d11_devices);

    loop {
        if !handle_message( &mut window ) {
            break;
        }
        unsafe{
            if let Some(device_context) = d11_devices._device_context.as_ref(){
                let array: [f32; 4] = [0.4,0.8,0.0,1.0];
                device_context.ClearRenderTargetView(d11_devices._render_target, &array);
                
            }
            if let Some(swap_chain) = d11_devices._swap_chain.as_ref(){
                swap_chain.Present(0,0);
            }
        }
    }
}