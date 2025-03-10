use inline_spirv::inline_spirv;

pub const HELLO_TRIANGLE_VERT_SHADER: &'static [u32] = inline_spirv!(
    r#"
#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 outColor;

void main() {
	gl_Position = vec4(inPosition, 1.0);
	outColor = inColor;
}"#,
    vert
);

pub const HELLO_TRIANGLE_FRAG_SHADER: &'static [u32] = inline_spirv!(
    r#"
#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 0) out vec4 outColor;

void main() {
	outColor = vec4(fragColor, 1.0);
}"#,
    frag
);
