use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_3d,
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::RenderGraph,
        RenderApp,
    },
};
use node::FXAANode;
use pipeline::FXAAPipeline;

mod node;
mod pipeline;

#[derive(Component, Clone)]
pub struct FXAA {
    pub enabled: bool,
}

impl ExtractComponent for FXAA {
    type Query = &'static Self;
    type Filter = With<Camera>;

    fn extract_component(item: QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}

const BLIT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3212361765411723543);

const FXAA_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2982361765441723543);

pub struct FXAAPlugin;
impl Plugin for FXAAPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, BLIT_SHADER_HANDLE, "blit.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, FXAA_SHADER_HANDLE, "fxaa.wgsl", Shader::from_wgsl);

        app.add_plugin(ExtractComponentPlugin::<FXAA>::default());

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
        render_app.init_resource::<FXAAPipeline>();

        let fxaa_node = FXAANode::new(&mut render_app.world);

        let mut binding = render_app.world.resource_mut::<RenderGraph>();
        let graph = binding.get_sub_graph_mut(core_3d::graph::NAME).unwrap();

        graph.add_node(FXAA_NODE, fxaa_node);

        graph
            .add_slot_edge(
                graph.input_node().unwrap().id,
                core_3d::graph::input::VIEW_ENTITY,
                FXAA_NODE,
                FXAANode::IN_VIEW,
            )
            .unwrap();

        graph
            .remove_node_edge(
                core_3d::graph::node::TONEMAPPING,
                core_3d::graph::node::UPSCALING,
            )
            .unwrap();

        graph
            .add_node_edge(core_3d::graph::node::TONEMAPPING, FXAA_NODE)
            .unwrap();

        // TODO, fxaa doesn't show if this is set
        //graph
        //    .add_node_edge(FXAA_NODE, core_3d::graph::node::UPSCALING)
        //    .unwrap();
    }
}

pub const FXAA_NODE: &str = "fxaa_node";
