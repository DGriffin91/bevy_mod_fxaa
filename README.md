# bevy_mod_fxaa

Bevy has FXAA post processing built in now. Use that instead: https://github.com/bevyengine/bevy/blob/main/examples/3d/fxaa.rs

Experimenting here with putting FXAA right after the main pass similar to MSAA. 

The `ldr_prepass` branch does this using a prepass when HDR is enabled on the camera. The result however has some additional artifacts in comparison to the non HDR version. (it seems this should be avoidable by using `reinhard_luminance` for tonemapping and inverting that)

The `tonemap_in_shader` branch applies tonemapping in the FXAA shader at every texture sample when HDR is enabled on the camera. The result however has some additional artifacts in comparison to the non HDR version.

## FXAA post processing for Bevy

![settings_lg](settings_lg.png)

Usage:
```rust
// Add FXAA plugin
app.add_plugin(FXAAPlugin) 
```

```rust
// Add FXAA component to camera:
commands
    .spawn(Camera3dBundle::default())
    .insert(FXAA::default());
```

Currently depends on bevy 0.9.0-dev.
