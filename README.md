# render-engine

This is a 3D rendering engine for the Rust programming language based on an existing Rust Vulkan library (Vulkano). It is relatively complete, but not actively maintained because I have abandoned it to switch to Ash from Vulkano.

## Screenshots & Videos
[Youtube video demonstrating what's possible with this library](https://www.youtube.com/watch?v=UrnSCpf_yw0)

![](https://raw.githubusercontent.com/cynic64/tests-render-engine/master/screenshots/lighting.png)
Lighting demo using Blinn-Phong shading with ambient, diffuse, specular and normal maps

![](https://raw.githubusercontent.com/cynic64/tests-render-engine/master/screenshots/base.png)
Simple mesh loading demo using normals to shade the surface

![](https://raw.githubusercontent.com/cynic64/tests-render-engine/master/screenshots/multipass.png)
Minimalist example showing the use of multiple passes: first a triangle is drawn, then another fragment shader is used to desaturate it.

## Use
Render-engine isn't on crates.io and depends on a set of [low-level Vulkano helpers](https://github.com/cynic64/re-ll), so you'll have to clone the files rather than pointing Cargo to the github repo and reference them in your Cargo.toml like this:
```toml
[dependencies]
render-engine = { path = "../render-engine/" }
```

Render-engine requires re-ll, the set of low-level helpers, to be in the same directory as it as well as named `re-ll`. Eventually I will combine this repository, the helpers and the examples into one repository to make this process less painful.

## What's the point?
There are not yet any high-level rendering libraries for Rust, and especially not for Vulkan. The intent of this project was to fill that gap, and although I am abandoning it I still hope to eventually achieve what I wanted to with a new library built using lower-level Vulkan bindings (Ash). It was still a lot of fun and I learned a lot, so no regrets.

Although it's not feature-complete, it has fulfilled its purpose of being higher-level than existing rust libraries. The [triangle example](https://github.com/cynic64/tests-render-engine/blob/master/src/bin/triangle.rs) is 103 lines including whitespace, compared to [474](https://github.com/vulkano-rs/vulkano-examples/blob/master/src/bin/triangle.rs) for Vulkano and [1186](https://github.com/SaschaWillems/Vulkan/blob/master/examples/triangle/triangle.cpp) for Vulkan in C++. Lines of code is not a great metric, but the difference is clear anyway.

## Is it useable?
Yes! See the [examples repository](https://github.com/cynic64/tests-render-engine) and the previous youtube video for a demo of what can be done. Render-engine supports:
  - User-defined vertex types and polygon fill modes
  - Multiple passes
  - Multisampled anti-aliasing
  - Shaders loaded at runtime
  - Uniforms, both textures and pure data
  - All image formats supported by Vulkan

It also includes a custom input handling library for things like mouse movement and keypresses to make life easier (it's a layer on top of winit).
That said, it's very unfinished and definitely not to be used for anything more than hobby projects. Performance is a bit worse than it should be compared to raw Vulkan and some basic features like mipmaps aren't supported.

## Documentation
It doesn't exist (there are comments, but not enough for someone else to easily understand the codebase). The point of this was for me to learn, and because the structure of the library was constantly changing anyway I didn't write any documentation

## Why switch from Vulkano?
Vulkano is an ambitious library that has some really cool features and I appreciate the work of those who made it even more after creating my own rendering engine. However, the lack of support for certain features (mipmaps or constructing framebuffers dynamically, for example) as well as the lack of documentation when it comes to anything more complicated than rendering a simple 3D model (using multiple queues, for example) were starting to bother me.
I've started working on a new rendering engine with ash (not yet uploaded to github) and so far I really enjoy it, especially how the existing Vulkan docs transfer pretty much directly, meaning there are automatically a huge number of tutorials and solutions available for it.
