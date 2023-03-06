#[cfg(windows)]
extern crate winapi;
use std::ffi::CString;
use std::io::Error;
use std::mem;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping;
use std::ptr::null_mut;

use crate::winapi::Interface;
use winapi::shared::dxgi::*;
use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::*;
use winapi::shared::minwindef::{LPARAM, LPVOID, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HICON, HWND, RECT};
use winapi::shared::winerror::FAILED;
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::{
    ID3DBlob, D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D_DRIVER_TYPE_HARDWARE,
};
use winapi::um::d3dcompiler::*;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winnt::HRESULT;
use winapi::um::winuser::*;

pub mod vertex;

fn win32_string(value: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

struct Window {
    handle: HWND,
    width: i32,
    height: i32,
}

#[allow(dead_code)]
struct ConstantBufferStruct {
    model_view_projection: directx_math::XMFLOAT4X4,
}

struct Buffers {
    _vertex_buffer: *mut ID3D11Buffer,
    _index_buffer: *mut ID3D11Buffer,
    _constant_buffer: *mut ID3D11Buffer,
}

struct D11Devices {
    _device: *mut ID3D11Device,
    _device_context: *mut ID3D11DeviceContext,
    _swap_chain: *mut IDXGISwapChain,
    _back_buffer: *mut ID3D11Texture2D,
    _render_target: *mut ID3D11RenderTargetView,
}

fn handle_message() -> bool {
    unsafe {
        let mut message: MSG = mem::zeroed();
        if PeekMessageA(&mut message as *mut MSG, null_mut(), 0, 0, PM_REMOVE) > 0 {
            TranslateMessage(&message as *const MSG);
            DispatchMessageW(&message as *const MSG);
            if message.message == WM_QUIT {
                return false;
            }
        }
    }
    true
}

fn create_window(name: &str, title: &str) -> Result<Window, Error> {
    //Convert strings to correct format
    let name = win32_string(name);
    let title = win32_string(title);

    unsafe {
        let hinstance = winapi::um::libloaderapi::GetModuleHandleW(null_mut());
        let wnd_class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            hIconSm: 0 as HICON,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance,
            lpszClassName: name.as_ptr(),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
        };

        RegisterClassExW(&wnd_class);

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
            null_mut(),
        );

        let mut rect: RECT = mem::zeroed();
        GetClientRect(handle, &mut rect);
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        if handle.is_null() {
            Err(Error::last_os_error())
        } else {
            Ok(Window {
                handle,
                width,
                height,
            })
        }
    }
}

fn create_device(devices: &mut D11Devices) {
    #[cfg(debug_assertions)]
    let creation_flags = D3D11_CREATE_DEVICE_DEBUG;

    #[cfg(not(debug_assertions))]
    let creation_flags = 0;

    // Create Device and context
    unsafe {
        D3D11CreateDevice(
            null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            null_mut(),
            creation_flags,
            null_mut(),
            0,
            7,
            &mut devices._device,
            null_mut(),
            &mut devices._device_context,
        );
    }
}

fn create_swap_chain(window: &Window, devices: &mut D11Devices) {
    unsafe {
        // Describe the swap chain
        let mut swap_chain_desc: DXGI_SWAP_CHAIN_DESC = mem::zeroed();
        swap_chain_desc.BufferDesc.Width = window.width as u32;
        swap_chain_desc.BufferDesc.Height = window.height as u32;
        swap_chain_desc.BufferCount = 1;
        swap_chain_desc.Windowed = 1;
        swap_chain_desc.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
        swap_chain_desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
        swap_chain_desc.SampleDesc.Count = 1;
        swap_chain_desc.SampleDesc.Quality = 0;
        swap_chain_desc.SwapEffect = DXGI_SWAP_EFFECT_DISCARD; // TODO: Change this. DXGI_SWAP_EFFECT_FLIP_DISCARD and use BufferCount = 2
        swap_chain_desc.OutputWindow = window.handle;

        let mut dxgi_device: *mut IDXGIDevice = null_mut();
        let mut dxgi_adapter: *mut IDXGIAdapter = null_mut();
        let mut dxgi_factory: *mut IDXGIFactory1 = null_mut();

        // get dxgi device
        devices._device.as_ref().unwrap().QueryInterface(
            &IDXGIDevice::uuidof(),
            &mut dxgi_device as *mut *mut IDXGIDevice as *mut *mut winapi::ctypes::c_void,
        );

        // Get dxgi adapter
        dxgi_device.as_ref().unwrap().GetAdapter(&mut dxgi_adapter);

        // Get dxgi factory
        dxgi_adapter.as_ref().unwrap().GetParent(
            &IDXGIFactory1::uuidof(),
            &mut dxgi_factory as *mut *mut IDXGIFactory1 as *mut *mut winapi::ctypes::c_void,
        );

        // Create SwapChain
        dxgi_factory.as_ref().unwrap().CreateSwapChain(
            devices._device as *mut IUnknown,
            &mut swap_chain_desc,
            &mut devices._swap_chain,
        );

        // Get swap chain’s back buffer
        devices._swap_chain.as_ref().unwrap().GetBuffer(
            0,
            &IID_ID3D11Texture2D,
            &mut devices._back_buffer as *mut _ as *mut LPVOID,
        );
        //  Create the render target view
        devices._device.as_ref().unwrap().CreateRenderTargetView(
            devices._back_buffer as *mut _,
            null_mut(),
            &mut devices._render_target as *mut _ as *mut _,
        );

        // Bind views.
        // TODO - DepthStencilView (Depth Buffer)
        devices
            ._device_context
            .as_ref()
            .unwrap()
            .OMSetRenderTargets(1, &mut devices._render_target as _, null_mut());
    }
}

fn set_viewport(window: &Window, devices: &D11Devices) {
    unsafe {
        let mut viewport: D3D11_VIEWPORT = mem::zeroed();
        viewport.TopLeftX = 0.0;
        viewport.TopLeftY = 0.0;
        viewport.Width = window.width as f32;
        viewport.Height = window.height as f32;
        viewport.MinDepth = 0.0;
        viewport.MaxDepth = 1.0;

        devices
            ._device_context
            .as_ref()
            .unwrap()
            .RSSetViewports(1, &viewport);
    }
}

#[allow(temporary_cstring_as_ptr)]
fn init_pipeline(devices: &D11Devices) {
    unsafe {
        let mut vertex_shader: *mut ID3DBlob = mem::zeroed();
        let mut pixel_shader: *mut ID3DBlob = mem::zeroed();

        // TODO move to struct?
        let mut p_vs: *mut ID3D11VertexShader = null_mut();
        let mut p_ps: *mut ID3D11PixelShader = null_mut();
        let mut p_layout: *mut ID3D11InputLayout = mem::zeroed();

        // Compile Vertex Shader
        let res: HRESULT = D3DCompileFromFile(
            win32_string("shaders.hlsl").as_ptr(), // Convert to correct format LPCWSTR
            std::ptr::null(),
            D3D_COMPILE_STANDARD_FILE_INCLUDE,
            String::from("VSMain").into_bytes().as_ptr() as *const i8, // Convert to correct format. LPCSTR
            CString::new("vs_5_0").unwrap().as_ptr() as *const i8, // Convert to correct format. LPCSTR. TODO: why string::from doesn't work?
            D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION,
            0,
            &mut vertex_shader,
            null_mut(),
        );
        if FAILED(res) {
            println!("Error Compiling Vertex Shader: {}", res)
        }

        // Compile Pixel Shader
        let res: HRESULT = D3DCompileFromFile(
            win32_string("shaders.hlsl").as_ptr(), //Convert to correct char format LPCWSTR
            std::ptr::null(),
            D3D_COMPILE_STANDARD_FILE_INCLUDE,
            String::from("PSMain").into_bytes().as_ptr() as *const i8, //Convert to correct char format. LPCSTR
            CString::new("ps_5_0").unwrap().as_ptr() as *const i8, //Convert to correct char format. LPCSTR. TODO: why string::from doesn't work?
            D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION,
            0,
            &mut pixel_shader,
            null_mut(),
        );
        if FAILED(res) {
            println!("Error Compiling Pixel Shader: {}", res)
        }

        // Create Vertex shader
        let res = devices._device.as_ref().unwrap().CreateVertexShader(
            vertex_shader.as_ref().unwrap().GetBufferPointer(),
            vertex_shader.as_ref().unwrap().GetBufferSize(),
            null_mut(),
            &mut p_vs,
        );
        if FAILED(res) {
            println!("Error creating Vertex Shader: {}", res)
        }

        // Create Pixel shader
        let res = devices._device.as_ref().unwrap().CreatePixelShader(
            pixel_shader.as_ref().unwrap().GetBufferPointer(),
            pixel_shader.as_ref().unwrap().GetBufferSize(),
            null_mut(),
            &mut p_ps,
        );
        if FAILED(res) {
            println!("Error creating Pixel Shader: {}", res)
        }

        // Set the shader objects
        devices
            ._device_context
            .as_ref()
            .unwrap()
            .VSSetShader(p_vs, null_mut(), 0);
        devices
            ._device_context
            .as_ref()
            .unwrap()
            .PSSetShader(p_ps, null_mut(), 0);

        // Create the input layout object
        // TODO: WTF???
        let c_position = CString::new("POSITION").unwrap();
        let c_color = CString::new("COLOR").unwrap();
        let local_layout = [
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: c_position.as_ptr() as *const i8,
                //SemanticName: CString::new("POSITION").unwrap().as_ptr() as *const i8, // This doesn't work
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 0,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: c_color.as_ptr() as *const i8,
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 16,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
        ];

        let res = devices._device.as_ref().unwrap().CreateInputLayout(
            local_layout.as_ptr(),
            local_layout.len() as _,
            vertex_shader.as_ref().unwrap().GetBufferPointer(),
            vertex_shader.as_ref().unwrap().GetBufferSize(),
            &mut p_layout,
        );
        if FAILED(res) {
            println!("Error creating Input Layout: {}", res)
        }

        // Set the inpput layout object
        devices
            ._device_context
            .as_ref()
            .unwrap()
            .IASetInputLayout(p_layout);
    }
}

fn init_graphics(devices: &D11Devices, buffers: &mut Buffers) {
    let triangle_verticles = [
        vertex::Vertex {
            pos: directx_math::XMFLOAT4 {
                x: -1.0,
                y: 1.0,
                z: 0.0,
                w: 1.0,
            },
            color: directx_math::XMFLOAT4 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
        },
        vertex::Vertex {
            pos: directx_math::XMFLOAT4 {
                x: 1.0,
                y: -1.0,
                z: 0.0,
                w: 1.0,
            },
            color: directx_math::XMFLOAT4 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
                w: 1.0,
            },
        },
        vertex::Vertex {
            pos: directx_math::XMFLOAT4 {
                x: -1.0,
                y: -1.0,
                z: 0.0,
                w: 1.0,
            },
            color: directx_math::XMFLOAT4 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
                w: 1.0,
            },
        },
        vertex::Vertex {
            pos: directx_math::XMFLOAT4 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
                w: 1.0,
            },
            color: directx_math::XMFLOAT4 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
                w: 1.0,
            },
        },
    ];

    let triangle_indices = [0, 1, 2, 3, 1, 0];

    // Describe the vertex buffer
    let buffer_desc = D3D11_BUFFER_DESC {
        Usage: D3D11_USAGE_DYNAMIC,
        ByteWidth: mem::size_of::<vertex::Vertex>() as u32 * triangle_verticles.len() as u32,
        BindFlags: D3D11_BIND_VERTEX_BUFFER,
        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
        MiscFlags: 0,
        StructureByteStride: 0,
    };

    // Describe the index buffer
    let index_buffer_desc = D3D11_BUFFER_DESC {
        Usage: D3D11_USAGE_IMMUTABLE,
        ByteWidth: (mem::size_of::<UINT>() * triangle_indices.len()) as u32,
        BindFlags: D3D11_BIND_INDEX_BUFFER,
        CPUAccessFlags: 0,
        MiscFlags: 0,
        StructureByteStride: 0,
    };

    unsafe {
        // Create Vertex Buffer
        let res = devices._device.as_ref().unwrap().CreateBuffer(
            &buffer_desc,
            null_mut(),
            &mut buffers._vertex_buffer,
        );
        if FAILED(res) {
            panic!("Error creating buffer: {}", res)
        }

        // Copy the vertices into the buffer
        let mut ms: D3D11_MAPPED_SUBRESOURCE = mem::zeroed();
        devices._device_context.as_ref().unwrap().Map(
            buffers._vertex_buffer as _,
            0,
            D3D11_MAP_WRITE_DISCARD,
            0,
            &mut ms,
        );
        copy_nonoverlapping(&triangle_verticles, ms.pData as _, triangle_verticles.len());
        devices
            ._device_context
            .as_ref()
            .unwrap()
            .Unmap(buffers._vertex_buffer as _, 0);

        // Create Index buffer
        // Specify the data to initialize the index buffer.
        let iinit_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: triangle_indices.as_ptr() as _,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        // Create the index buffer.
        devices._device.as_ref().unwrap().CreateBuffer(
            &index_buffer_desc,
            &iinit_data,
            &mut buffers._index_buffer,
        );

        // select which vertex buffer to use
        let stride = mem::size_of::<vertex::Vertex>() as u32;
        let offset = 0;
        devices
            ._device_context
            .as_ref()
            .unwrap()
            .IASetVertexBuffers(0, 1, &buffers._vertex_buffer, &stride, &offset);

        // select which index buffer to use
        devices._device_context.as_ref().unwrap().IASetIndexBuffer(
            buffers._index_buffer,
            DXGI_FORMAT_R32_UINT,
            0,
        );

        // select which primtive type we are using
        devices
            ._device_context
            .as_ref()
            .unwrap()
            .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
    }
}

fn init_constant_buffer(devices: &D11Devices, buffers: &mut Buffers) {
    unsafe {
        // Describe constant buffer
        let buffer_desc = D3D11_BUFFER_DESC {
            Usage: D3D11_USAGE_DYNAMIC,
            ByteWidth: 128, // THIS IS IMPORTANT!
            BindFlags: D3D11_BIND_CONSTANT_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
            MiscFlags: 0,
            StructureByteStride: 0,
        };

        // Create Buffer
        devices._device.as_ref().unwrap().CreateBuffer(
            &buffer_desc,
            null_mut(),
            &mut buffers._constant_buffer,
        );

        // Set Constant buffer
        devices
            ._device_context
            .as_ref()
            .unwrap()
            .VSSetConstantBuffers(0, 1, &buffers._constant_buffer);
    }
}

fn set_constant_buffer(f: &f32, window: &Window, devices: &D11Devices, buffers: &mut Buffers) {
    unsafe {
        let mut model_view_projection: directx_math::XMFLOAT4X4 = mem::zeroed();

        // Create Orthographic projection matrix. To create perspective use: directx_math::XMMatrixPerspectiveFovLH(0.25 * directx_math::XM_PI, aspect, 0.1, 50.0)
        let aspect = window.width as f32 / window.height as f32;
        let projection = directx_math::XMMatrixOrthographicLH(aspect, 1.0, 0.01, 50.0);

        // Create view matrix
        let eye_position = directx_math::XMVectorSet(0.0, 0.0, -10.0, 1.0);
        let focus_position = directx_math::XMVectorSet(0.0, 0.0, 0.0, 1.0);
        let up_direction = directx_math::XMVectorSet(0.0, 1.0, 0.0, 1.0);
        let view = directx_math::XMMatrixLookAtLH(eye_position, focus_position, up_direction);

        // Create Model Matrix
        // Scale first, then rotate, then move
        let scale = directx_math::XMMatrixScaling(0.25, 0.25, 0.25);
        let rotation = directx_math::XMMatrixRotationRollPitchYaw(*f, *f, 0.0);
        let transform = directx_math::XMMatrixTranslation(0.25, 0.0, 0.0);
        let mut model = directx_math::XMMatrixMultiply(scale, &rotation);
        model = directx_math::XMMatrixMultiply(model, &transform);

        // Create model_view_projection matrix
        let mut model_view_projection_matrix = directx_math::XMMatrixMultiply(model, &view);
        model_view_projection_matrix =
            directx_math::XMMatrixMultiply(model_view_projection_matrix, &projection);

        // Create model_view_projection constant
        // XMMatrixTranspose is very important! Read Remarks: https://learn.microsoft.com/en-us/windows/win32/api/directxmath/nf-directxmath-XMStoreFloat4x4
        directx_math::XMStoreFloat4x4(
            &mut model_view_projection,
            directx_math::XMMatrixTranspose(model_view_projection_matrix),
        );
        let constant_buffer = ConstantBufferStruct {
            model_view_projection,
        };

        // Copy the constant data into the buffer
        let mut ms: D3D11_MAPPED_SUBRESOURCE = mem::zeroed();
        devices._device_context.as_ref().unwrap().Map(
            buffers._constant_buffer as _,
            0,
            D3D11_MAP_WRITE_DISCARD,
            0,
            &mut ms,
        );
        copy_nonoverlapping(&constant_buffer, ms.pData as _, 1);
        devices
            ._device_context
            .as_ref()
            .unwrap()
            .Unmap(buffers._constant_buffer as _, 0);
    }
}

fn main() {
    let name = "winclass1";
    let title = "win_title";

    let mut d11_devices = D11Devices {
        _swap_chain: unsafe { mem::zeroed() },
        _device: unsafe { mem::zeroed() },
        _device_context: unsafe { mem::zeroed() },
        _back_buffer: unsafe { mem::zeroed() },
        _render_target: unsafe { mem::zeroed() },
    };

    let mut buffers = Buffers {
        _vertex_buffer: unsafe { mem::zeroed() },
        _constant_buffer: unsafe { mem::zeroed() },
        _index_buffer: unsafe { mem::zeroed() },
    };

    // 1. Create window
    // 2. Create Device and context
    // 3. Create Swap Chain
    // 4. Set viewport
    // 5. Init Pipeline
    // 6. Init Graphics
    // 7. Init Constant buffer
    let mut f = 0.0;
    let window = create_window(name, title).unwrap();
    create_device(&mut d11_devices);
    create_swap_chain(&window, &mut d11_devices);
    set_viewport(&window, &d11_devices);
    init_pipeline(&d11_devices);
    init_graphics(&d11_devices, &mut buffers);
    init_constant_buffer(&d11_devices, &mut buffers);

    loop {
        if !handle_message() {
            break;
        }
        unsafe {
            // Clear Canvas
            let array: [f32; 4] = [0.1, 0.0, 0.3, 1.0];
            d11_devices
                ._device_context
                .as_ref()
                .unwrap()
                .ClearRenderTargetView(d11_devices._render_target, &array);

            f += 0.001;
            set_constant_buffer(&f, &window, &d11_devices, &mut buffers);

            // draw the vertex buffer to the back buffer
            d11_devices
                ._device_context
                .as_ref()
                .unwrap()
                .DrawIndexed(6, 0, 0);

            // Switch back & front buffers
            d11_devices._swap_chain.as_ref().unwrap().Present(0, 0);
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match u_msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            return 0;
        }
        _ => 0,
    };
    DefWindowProcA(hwnd, u_msg, w_param, l_param)
}
