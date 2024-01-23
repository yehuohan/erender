//! 场景管理
//!
//! 一个Scene包含了Model, Camera, Light等；
//! Model是对外的模型渲染单位，需要渲染的模型应当放到Model里面；
//! Mesh是内部渲染单位，一个Model可包含多个Mesh，渲染Model时，依次渲染其中的Mesh。

pub mod camera;
pub mod light;
pub mod model;
pub mod scene;
