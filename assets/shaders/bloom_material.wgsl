fn jodieReinhardTonemap(c: vec3<f32>) -> vec3<f32> {
    let l: f32 = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
    let tc: vec3<f32> = c / (c + 1.);

    return mix(c / (l + 1.), tc, tc);
}

fn bloomTile(lod: f32, offset: vec2<f32>, uv: vec2<f32>) -> vec3<f32> {
    return sample_texture(BUFFER_iChannel1, uv * exp2(-lod) + offset).rgb;
}

fn getBloom(uv: vec2<f32>) -> vec3<f32> {
    var blur: vec3<f32> = vec3<f32>(0.);

    blur = pow(bloomTile(2., vec2<f32>(0., 0.), uv), vec3<f32>(2.2)) + blur;
    blur = pow(bloomTile(3., vec2<f32>(0.3, 0.), uv), vec3<f32>(2.2)) * 1.3 + blur;
    blur = pow(bloomTile(4., vec2<f32>(0., 0.3), uv), vec3<f32>(2.2)) * 1.6 + blur;
    blur = pow(bloomTile(5., vec2<f32>(0.1, 0.3), uv), vec3<f32>(2.2)) * 1.9 + blur;
    blur = pow(bloomTile(6., vec2<f32>(0.2, 0.3), uv), vec3<f32>(2.2)) * 2.2 + blur;

    return blur * 24.;
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let R: vec2<f32> = uni.iResolution.xy;
    let y_inverted_location = vec2<i32>(i32(invocation_id.x), i32(R.y) - i32(invocation_id.y));
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    var fragColor: vec4<f32>;
    var fragCoord = vec2<f32>(f32(location.x), f32(location.y) );

    let uv: vec2<f32> = fragCoord.xy / uni.iResolution.xy;
    var color: vec3<f32> = pow(sample_texture(BUFFER_iChannel0, uv).rgb * 24., vec3<f32>(2.2));
    color = pow(color, vec3<f32>(2.2));
    color = color + (pow(getBloom(uv), vec3<f32>(2.2)));
    color = pow(color, vec3<f32>(1. / 2.2));
    color = jodieReinhardTonemap(color);

    fragColor = vec4<f32>(color, 1.);
}

//#define colorRange 24.0
//
//layout(set = 2, binding = 0) uniform vec4 CustomMaterial_color;
//
//vec3 jodieReinhardTonemap(vec3 c)
//{
//    float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
//    vec3 tc = c / (c + 1.0);
//
//    return mix(c / (l + 1.0), tc, tc);
//}
//
//vec3 bloomTile(float lod, vec2 offset, vec2 uv)
//{
//    return texture(CustomMaterial_color, uv * exp2(-lod) + offset).rgb;
//}
//
//vec3 getBloom(vec2 uv)
//{
//    vec3 blur = vec3(0.0);
//
//    blur = pow(bloomTile(2., vec2(0.0,0.0), uv),vec3(2.2))       	   	+ blur;
//    blur = pow(bloomTile(3., vec2(0.3,0.0), uv),vec3(2.2)) * 1.3        + blur;
//    blur = pow(bloomTile(4., vec2(0.0,0.3), uv),vec3(2.2)) * 1.6        + blur;
//    blur = pow(bloomTile(5., vec2(0.1,0.3), uv),vec3(2.2)) * 1.9 	   	+ blur;
//    blur = pow(bloomTile(6., vec2(0.2,0.3), uv),vec3(2.2)) * 2.2 	   	+ blur;
//
//    return blur * colorRange;
//}
//
//void mainImage( out vec4 fragColor, in vec2 fragCoord )
//{
//	vec2 uv = fragCoord.xy / iResolution.xy;
//
//    vec3 color = pow(texture(CustomMaterial_color, uv).rgb * colorRange, vec3(2.2));
//    color = pow(color, vec3(2.2));
//    color += pow(getBloom(uv), vec3(2.2));
//    color = pow(color, vec3(1.0 / 2.2));
//
//    color = jodieReinhardTonemap(color);
//
//	fragColor = vec4(color, 1.0);
//}

