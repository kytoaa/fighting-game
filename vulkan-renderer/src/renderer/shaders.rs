use inline_spirv::inline_spirv;

pub const VERT_SHADER: &'static [u32] = inline_spirv!(
    r#"
#version 450

layout(binding = 0) uniform Matrices {
    mat4 proj;
} matrices;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec2 inUV;
layout(location = 2) in uint inTextureIndex;

layout(location = 0) out vec2 outUV;
layout(location = 1) out uint outTextureIndex;

void main() {
	gl_Position = matrices.proj * vec4(inPosition, 1.0) * vec4(6.0, 6.0, 1.0, 1.0);
	outUV = inUV;
    outTextureIndex = inTextureIndex;
}"#,
    vert
);

pub const DRAW_SPRITE_FRAG_SHADER: &'static [u32] = inline_spirv!(
    r#"
#version 450

layout(set = 1, binding = 0) uniform sampler2D textureSamplers[64];

layout(location = 0) in vec2 fragUV;
layout(location = 1) flat in uint inTextureIndex;

layout(location = 0) out vec4 outColor;

void main() {
    if (inTextureIndex < 64) {
        outColor = texture(textureSamplers[inTextureIndex], fragUV);

        if (outColor.a < 0.5) {
            discard;
        }
    } else {
        switch (inTextureIndex) {
            case 64:
                outColor = vec4(1.0, 0.2, 0.2, 1.0);
                return;
            case 65:
                outColor = vec4(0.2, 1.0, 0.2, 1.0);
                return;
            case 66:
                outColor = vec4(0.2, 1.0, 0.2, 1.0);
                return;
            default:
                outColor = vec4(1.0, 0.0, 1.0, 1.0);
                return;
        }
    }
}"#,
    frag
);
