#import bevy_sprite::mesh2d_view_bindings::globals
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_functions::mesh2d_position_world_to_clip

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> left: vec2<f32>;
@group(2) @binding(2) var<uniform> right: vec2<f32>;
@group(2) @binding(5) var<uniform> mesh_size: vec4<f32>; // Must be vec4 to align to 16 bit (wasm)

const blue_shift: vec4<f32> = vec4<f32>(1., 1., 1., 0.);

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV coordinates to a coordinate system that matches your original logic
    // This transforms from [0,1] to [-mesh_size/2, mesh_size/2]
    let uv_coords = (in.uv - vec2<f32>(0.5, 0.5)) * mesh_size.xy;

    var c0: f32 = 0.;
    var c1: f32 = 0.;

    for (var i = 0u; i < 25; i += 1u) {
        var t: f32 = 4. * globals.time + hash11(f32(i));
        var v: vec2<f32> = hash21(f32(i) + 50. * floor(t));
        t = fract(t);
        v = vec2<f32>(sqrt(-2. * log(1. - v.x)), 6.283185 * v.y);
        v = 50. * v.x * vec2<f32>(cos(v.y), sin(v.y));

        let bound: vec2<f32> = (right - left);
        let bound_d: f32 = length(bound);

        // Use UV-based coordinates instead of world_position
        let point1: vec2<f32> = uv_coords - left;
        let point2: vec2<f32> = uv_coords - right;

        let d1: f32 = length(point1);
        let d2: f32 = length(point2);
        let cos1_p: f32 = dot(point1, bound) / (bound_d * d1);
        let cos2_p: f32 = dot(point2, bound) / (bound_d * d2);

        var pos: vec2<f32>;
        if (cos1_p >= 0. && cos2_p <= 0.) {
            let proj_t: f32 = dot(point1, bound) / dot(bound, bound);
            pos = vec2<f32>(left.x + proj_t * bound.x, left.y + proj_t * bound.y);
        } else {
            if (d2 > d1) {
                pos = left;
            } else {
                pos = right;
            }
        }

        // Use uv_coords instead of world_position
        var p: vec2<f32> = pos - uv_coords + t * v;
        c0 = c0 + (4. * (1. - t) / (1. + 0.3 * dot(p, p)));
        p = p.yx;
        v = v.yx;
        p = vec2<f32>(p.x / v.x, p.y - p.x / v.x * v.y);

        var a: f32 = 0.;
        if (abs(p.x) < 0.1) { a = 50. / abs(v.x); } else { a = 0.; };

        let b0: f32 = max(2. - abs(p.y), 0.);
        let b1: f32 = max(2. - abs(p.y), 0.);
        c0 = c0 + ((1. - t) * b0 * a);
        c1 = c1 + ((1. - t) * b1 * a);
    }

    var rgb: vec4<f32> = c0 * color + c1 * color * blue_shift;
    // Use uv_coords for noise to make it stick to the mesh
    rgb = vec4(rgb.xyz + (hash33(vec3<f32>(uv_coords.xy, globals.time * 256.)) / 512.), rgb.w);
    rgb = pow(rgb, vec4<f32>(1.));

    return rgb;
}

fn pcg(v: u32) -> u32 {
	let state = v * 747796405u + 2891336453u;
	let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
	return (word >> 22u) ^ word;
}

fn pcg2d(v: vec2<u32>) -> vec2<u32> {
  	var nv = v;

	nv = nv * 1664525u + 1013904223u;
	nv.x = nv.x + (nv.y * 1664525u);
	nv.y = nv.y + (nv.x * 1664525u);

	nv = nv ^ (nv >> vec2(16u));
	nv.x = nv.x + (nv.y * 1664525u);
	nv.y = nv.y + (nv.x * 1664525u);

	nv = nv ^ (nv >> vec2(16u));

	return nv;
}

fn pcg3d(v: vec3<u32>) -> vec3<u32> {
	var nv = v;

	nv = nv * 1664525u + 1013904223u;
	nv.x = nv.x + (nv.y * nv.z);
	nv.y = nv.y + (nv.z * nv.x);
	nv.z = nv.z + (nv.x * nv.y);

	nv = nv ^ (nv >> vec3(16u));
	nv.x = nv.x + (nv.y * nv.z);
	nv.y = nv.y + (nv.z * nv.x);
	nv.z = nv.z + (nv.x * nv.y);

	return nv;
}

fn hash11(p: f32) -> f32 {
	return f32(pcg(u32(p))) / 4294967300.;
}

fn hash21(p: f32) -> vec2<f32> {
	return vec2<f32>(pcg2d(vec2<u32>(u32(p), 0u))) / 4294967300.;
}

fn hash33(p3: vec3<f32>) -> vec3<f32> {
	return vec3<f32>(pcg3d(vec3<u32>(p3))) / 4294967300.;
}