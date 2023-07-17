pub use wasmtime::*;

#[cfg(feature = "component-model")]
mod universal_component;

#[cfg(feature = "component-model-no-universal")]
pub mod component {
    pub use wasmtime::component::*;

    pub use wasm_bridge_macros::bindgen_sys as bindgen;
    pub use wasm_bridge_macros::flags;
    pub use wasm_bridge_macros::ComponentType;
    pub use wasm_bridge_macros::Lift;
    pub use wasm_bridge_macros::Lower;

    #[cfg(feature = "component-model")]
    pub use super::universal_component::new_universal_component;
}
