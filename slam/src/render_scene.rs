use glam::Mat4;
use glow::HasContext;
use std::rc::Rc;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex { pos: [f32; 3], col: [f32; 4] }

const VS: &str = r#"#version 330 core
layout(location=0) in vec3 pos;
layout(location=1) in vec4 col;
uniform mat4 mvp;
out vec4 v_col;
void main() { gl_Position = mvp * vec4(pos, 1.0); gl_PointSize = 4.0; v_col = col; }"#;

const FS: &str = r#"#version 330 core
in vec4 v_col; out vec4 o;
void main() { o = v_col; }"#;

pub struct Scene3D {
	gl: Rc<glow::Context>,
	prog: glow::Program,
	mvp_loc: Option<glow::UniformLocation>,
	vao: glow::VertexArray,
	vbo: glow::Buffer,
	fbo: Option<glow::Framebuffer>,
	pub tex: Option<glow::Texture>,
	depth: Option<glow::Renderbuffer>,
	size: (i32, i32),
	points: Vec<Vertex>,
	lines: Vec<Vertex>,
	tris: Vec<Vertex>,
}

impl Scene3D {
	pub fn new(gl: Rc<glow::Context>) -> Self {
		unsafe {
			let prog = gl.create_program().unwrap();
			for (ty, src) in [(glow::VERTEX_SHADER, VS), (glow::FRAGMENT_SHADER, FS)] {
				let sh = gl.create_shader(ty).unwrap();
				gl.shader_source(sh, src);
				gl.compile_shader(sh);
				assert!(gl.get_shader_compile_status(sh), "{}", gl.get_shader_info_log(sh));
				gl.attach_shader(prog, sh);
			}
			gl.link_program(prog);
			assert!(gl.get_program_link_status(prog), "{}", gl.get_program_info_log(prog));
			let mvp_loc = gl.get_uniform_location(prog, "mvp");
			
			let vao = gl.create_vertex_array().unwrap();
			let vbo = gl.create_buffer().unwrap();
			gl.bind_vertex_array(Some(vao));
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
			let stride = std::mem::size_of::<Vertex>() as i32;
			gl.enable_vertex_attrib_array(0);
			gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
			gl.enable_vertex_attrib_array(1);
			gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, stride, 12);
			gl.bind_vertex_array(None);
			
			Self { gl, prog, mvp_loc, vao, vbo, fbo: None, tex: None, depth: None,
			size: (0, 0), points: vec![], lines: vec![], tris: vec![] }
		}
	}
	
	// --- immediate-mode API: call these every frame ---
	#[allow(dead_code)]
	pub fn point(&mut self, p: [f32; 3], c: [f32; 4]) { self.points.push(Vertex { pos: p, col: c }); }
	#[allow(dead_code)]
	pub fn line(&mut self, a: [f32; 3], b: [f32; 3], c: [f32; 4]) {
		self.lines.push(Vertex { pos: a, col: c });
		self.lines.push(Vertex { pos: b, col: c });
	}
	#[allow(dead_code)]
	pub fn tri(&mut self, a: [f32; 3], b: [f32; 3], c3: [f32; 3], c: [f32; 4]) {
		for p in [a, b, c3] { self.tris.push(Vertex { pos: p, col: c }); }
	}
	
	fn resize(&mut self, w: i32, h: i32) {
		unsafe {
			let gl = &self.gl;
			if let Some(f) = self.fbo.take() { gl.delete_framebuffer(f); }
			if let Some(t) = self.tex.take() { gl.delete_texture(t); }
			if let Some(d) = self.depth.take() { gl.delete_renderbuffer(d); }
			
			let tex = gl.create_texture().unwrap();
			gl.bind_texture(glow::TEXTURE_2D, Some(tex));
			gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA8 as i32, w, h, 0,
			glow::RGBA, glow::UNSIGNED_BYTE, glow::PixelUnpackData::Slice(None));
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
			
			let depth = gl.create_renderbuffer().unwrap();
			gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth));
			gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH_COMPONENT24, w, h);
			
			let fbo = gl.create_framebuffer().unwrap();
			gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
			gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0,
			glow::TEXTURE_2D, Some(tex), 0);
			gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT,
			glow::RENDERBUFFER, Some(depth));
			gl.bind_framebuffer(glow::FRAMEBUFFER, None);
			
			self.fbo = Some(fbo); self.tex = Some(tex); self.depth = Some(depth);
			self.size = (w, h);
		}
	}
	
	pub fn render(&mut self, w: i32, h: i32, mvp: Mat4) {
		if (w, h) != self.size { self.resize(w, h); }
		unsafe {
			let gl = &self.gl;
			gl.bind_framebuffer(glow::FRAMEBUFFER, self.fbo);
			gl.viewport(0, 0, w, h);
			gl.clear_color(0.08, 0.08, 0.1, 1.0); // alpha 0.0 if you want it see-through
			gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
			gl.enable(glow::DEPTH_TEST);
			gl.enable(glow::PROGRAM_POINT_SIZE);
			gl.use_program(Some(self.prog));
			gl.uniform_matrix_4_f32_slice(self.mvp_loc.as_ref(), false, &mvp.to_cols_array());
			gl.bind_vertex_array(Some(self.vao));
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
			for (verts, mode) in [(&self.tris, glow::TRIANGLES),
			(&self.lines, glow::LINES),
			(&self.points, glow::POINTS)] {
				if verts.is_empty() { continue; }
				gl.buffer_data_u8_slice(glow::ARRAY_BUFFER,
				bytemuck::cast_slice(verts), glow::STREAM_DRAW);
				gl.draw_arrays(mode, 0, verts.len() as i32);
			}
			// put things back the way imgui's renderer expects
			gl.disable(glow::DEPTH_TEST);
			gl.bind_vertex_array(None);
			gl.use_program(None);
			gl.bind_framebuffer(glow::FRAMEBUFFER, None);
		}
		self.points.clear(); self.lines.clear(); self.tris.clear();
	}
}
