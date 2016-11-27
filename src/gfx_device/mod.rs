macro_rules! unwind_gfx_device_inner_mut {
    ($variable:expr, $field1:ident, $expr_field:expr, $expr_null:expr) => {
        match $variable {
            GfxDeviceInner::OpenGL {
                ref mut $field1,
                ..
            } => $expr_field,
            #[cfg(windows)]
            GfxDeviceInner::Direct3D { } => unimplemented!(),
            GfxDeviceInner::Null => $expr_null,
        }
    };
}

extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx;

use config::Element;
use std::path::Path;
use renderer;
use renderer::{Layer, Target, Light};
use ecs::{World, Component, VecStorage, Join};
use context::event::EngineEvent;
use processors::transform::LocalTransform;

mod gfx_device_inner;
mod texture;
mod mesh;
pub mod gfx_loader;
mod video_init;
pub use self::video_init::video_init;
use self::gfx_device_inner::GfxDeviceInner;
pub use self::texture::*;
pub use self::mesh::*;

config!(
    /// Contains display config,
    /// it is required to create a `VideoContext`
    struct DisplayConfig {
        pub title: String = "Amethyst game".to_string(),
        pub fullscreen: bool = false,
        pub dimensions: Option<(u32, u32)> = None,
        pub min_dimensions: Option<(u32, u32)> = None,
        pub max_dimensions: Option<(u32, u32)> = None,
        pub vsync: bool = true,
        pub multisampling: u16 = 1,
        pub visibility: bool = true,
        pub backend: String = "Null".to_string(),
    }
);

/// GfxDevice owns all resources related to graphics (e.g. amethyst_renderer::Renderer, gfx_device_gl::Device,
/// gfx_device_gl::Factory amethyst_renderer::Frame).
pub struct GfxDevice {
    gfx_device_inner: GfxDeviceInner,
}

impl GfxDevice {
    /// Create a new `GfxDevice` from `DisplayConfig`.
    pub fn new(gfx_device_inner: GfxDeviceInner) -> GfxDevice {
        GfxDevice { gfx_device_inner: gfx_device_inner }
    }

    /// Set the rendering pipeline to be used.
    pub fn set_pipeline(&mut self, _pipeline: Vec<Layer>) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            pipeline,
            {
                pipeline.layers = _pipeline;
            },
            ()
        )
    }

    /// Add a rendering `Target`.
    pub fn add_target(&mut self, target: Box<Target>, name: &str) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            pipeline,
            {
                pipeline.targets.insert(name.into(), target);
            },
            ()
        )
    }
    /// Delete a rendering `Target`.
    pub fn delete_target(&mut self, name: &str) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            pipeline,
            {
                pipeline.targets.remove(name.into());
            },
            ()
        )
    }

    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        match self.gfx_device_inner {
            GfxDeviceInner::OpenGL { ref window, .. } => window.get_inner_size(),
            #[cfg(windows)]
            GfxDeviceInner::Direct3D {} => unimplemented!(),
            GfxDeviceInner::Null => None,
        }
    }

    pub fn render_world(&mut self, world: &mut World) {
        match self.gfx_device_inner {
            GfxDeviceInner::OpenGL { ref mut renderer,
                                     ref mut device,
                                     ref pipeline,
                                     ref window,
                                     .. } => {
                let camera = world.read_resource::<renderer::Camera>().clone();
                let mut scene = renderer::Scene::<gfx_device_gl::Resources>::new(camera);
                let renderables = world.read::<Renderable>();
                let local_transforms = world.read::<LocalTransform>();
                for (renderable, local_transform) in (&renderables, &local_transforms).iter() {
                    let (buffer, slice) = match renderable.mesh.mesh_inner {
                        MeshInner::OpenGL { ref buffer,
                                            ref slice } => { (buffer.clone(), slice.clone()) },
                        _ => continue,
                        };
                    let ka = match renderable.ka.texture_inner {
                        TextureInner::OpenGL { ref texture } => texture.clone(),
                        _ => continue,
                    };
                    let kd = match renderable.kd.texture_inner {
                        TextureInner::OpenGL { ref texture } => texture.clone(),
                        _ => continue,
                    };
                    let transform = local_transform.matrix();
                    let fragment = renderer::Fragment {
                        transform: transform,
                        buffer: buffer,
                        slice: slice,
                        ka: ka,
                        kd: kd,
                    };
                    scene.fragments.push(fragment);
                }
                let lights = world.read::<Light>();
                for light in lights.iter() {
                    scene.lights.push(light.clone());
                }
                renderer.submit(pipeline, &scene, device);
                window.swap_buffers().unwrap();
            }
            #[cfg(windows)]
            GfxDeviceInner::Direct3D {} => unimplemented!(),
            GfxDeviceInner::Null => (),
        }
    }

    pub fn poll_events(&mut self) -> Vec<EngineEvent> {
        let mut events = vec![];
        match self.gfx_device_inner {
            GfxDeviceInner::OpenGL { ref window, .. } => {
                for event in window.poll_events() {
                    let event = EngineEvent::new(event);
                    events.push(event);
                }
            }
            #[cfg(windows)]
            GfxDeviceInner::Direct3D {} => {
                // stub
                unimplemented!();
            }
            GfxDeviceInner::Null => (),
        }
        events
    }
}

// pub enum MeshInner {
//     OpenGL {
//         buffer: gfx::handle::Buffer<gfx_device_gl::Resources, VertexPosNormal>,
//         slice: gfx::Slice<gfx_device_gl::Resources>,
//     },
//     #[cfg(windows)]
//     Direct3D {
//         // stub
//     },
//     Null,
// }

// pub struct Mesh {
//     pub mesh_inner: MeshInner,
// }

// pub enum TextureInner {
//     OpenGL {
//         texture: renderer::Texture<gfx_device_gl::Resources>,
//     },
//     #[cfg(windows)]
//     Direct3D {
//         // stub
//     },
//     Null,
// }

// pub struct Texture {
//     pub texture_inner: TextureInner,
// }

pub struct Renderable {
    pub mesh: Mesh,
    pub ka: Texture,
    pub kd: Texture,
}

impl Component for Renderable {
    type Storage = VecStorage<Renderable>;
}
