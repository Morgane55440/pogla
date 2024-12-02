use std::{
    ffi::{CStr, CString}, marker::PhantomData, ptr::{null, null_mut}
};

use anyhow::Result;

use gl::types::{GLchar, GLenum, GLfloat, GLint, GLsizei, GLuint, GLvoid};
use image::{ImageBuffer, Rgba};

macro_rules! check_err {
    () => {
        {
            let e = unsafe {gl::GetError() };
            assert_eq!(e, gl::NO_ERROR, "error code should be 0 but is : {:X?}", e)
        }
    };
}

pub struct Shader(GLuint);

impl Shader {
    const fn id(&self) -> GLuint {
        self.0
    }
    pub fn new(src : &str, shd_type : GLenum) -> Result<Self> {
        let mut compile_status= gl::TRUE.into();
        let shd = Shader(unsafe { gl::CreateShader(shd_type) });check_err!();
        let c_src =CString::new(src).unwrap();
        unsafe  {
            gl::ShaderSource(shd.id(), 1, (&c_src.as_ptr()) as *const _, null());check_err!();
            gl::CompileShader(shd.id());check_err!();
            gl::GetShaderiv(shd.id(), gl::COMPILE_STATUS, &mut compile_status);check_err!();
        }
        if compile_status != gl::TRUE.into() {
            Err(anyhow::Error::msg(unsafe { get_message_log(shd.id(), gl::GetShaderiv, gl::GetShaderInfoLog) }))
        } else {
            Ok(shd)
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id()) };check_err!();
    }
}

unsafe fn get_message_log(id : GLuint, len_fn : unsafe fn(GLuint, GLenum, *mut GLint), msg_fn : unsafe fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar)) -> String {
    let mut l = 0;
    unsafe {
        len_fn(id, gl::INFO_LOG_LENGTH, &mut l);
    }check_err!();

    let error = CString::new(" ".repeat(l as usize)).unwrap();

    unsafe {
        msg_fn(id, l, null_mut(), error.as_ptr() as *mut GLchar);
    }check_err!();
    error.to_string_lossy().to_string()

}

pub struct ShaderSrc<'a> {
    pub vertex_shader: &'a str,
    pub fragment_shader: &'a str,
    pub tessellation_control_shader: Option<&'a str>,
    pub tessellation_evaluation_shader: Option<&'a str>,
    pub geometry_shader: Option<&'a str>,
}

impl<'a> ShaderSrc<'a> {

    fn shader_iter(&self) -> impl Iterator<Item = (Result<Shader>, GLenum)> + '_ {
        [
            (Some(self.vertex_shader), gl::VERTEX_SHADER),
            (Some(self.fragment_shader), gl::FRAGMENT_SHADER),
            (self.tessellation_evaluation_shader, gl::TESS_EVALUATION_SHADER),
            (self.tessellation_control_shader, gl::TESS_CONTROL_SHADER),
            (self.geometry_shader, gl::GEOMETRY_SHADER)
        ].into_iter().filter_map(|(opt, shd_type)|Some((Shader::new(opt?, shd_type), shd_type)))
    }
}

struct ProgramId(GLuint);

impl ProgramId {
    const fn id(&self) -> GLuint {
        self.0
    }
    pub fn new() -> Result<Self> {
        let id = unsafe { gl::CreateProgram() };check_err!();
        if id == 0 { Err(anyhow::Error::msg("failed to create program"))} else {Ok(ProgramId(id))}
        
    }
}

impl Drop for ProgramId {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id()); };check_err!();
    }
}

pub struct Program{
    id : ProgramId,
    _shaders : Vec<Shader>,
}

impl Program {
    pub fn new(src : ShaderSrc<'_>) -> Result<Self> {
        let id = ProgramId::new()?;
        let _shaders : Vec<_> =  src.shader_iter().map(|(shd, _)| {
            let shd = shd?;
            unsafe { gl::AttachShader(id.id(), shd.id()) };check_err!();
            Ok(shd)
        }).collect::<Result<_>>()?;
        let mut linking_status = gl::TRUE.into();
        unsafe { 
            gl::LinkProgram(id.id());check_err!();
            gl::GetProgramiv(id.id(), gl::LINK_STATUS, &mut linking_status as *mut _);check_err!();
        }
        if linking_status != gl::TRUE.into() {
            Err(anyhow::Error::msg(unsafe { get_message_log(id.id(), gl::GetProgramiv, gl::GetProgramInfoLog) } ))
        } else {
            Ok(Program { id, _shaders })
        }
    }

    fn useprog(&self) {
        unsafe {gl::UseProgram(self.id());}check_err!();
    }

    const fn id(&self) -> GLuint {
        self.id.id()
    }
}

pub trait VertexDataType {
    fn element_type() -> GLenum;
    fn size() -> GLint;
    fn stride() -> GLsizei;
}

pub struct Vec3 {}

impl VertexDataType for Vec3 {
    fn element_type() -> GLenum {
        gl::FLOAT
    }

    fn size() -> GLint {
        3
    }

    fn stride() -> GLsizei {
        0
    }
}

pub struct Vec2 {}

impl VertexDataType for Vec2 {
    fn element_type() -> GLenum {
        gl::FLOAT
    }

    fn size() -> GLint {
        2
    }

    fn stride() -> GLsizei {
        0
    }
}

struct Vertices<T>{id : GLuint, _data : PhantomData<T>}

impl<T : VertexDataType> Vertices<T> {
    pub fn new(prog : &Program, data : &[GLfloat], location : &CStr) -> Result<Self> {
        prog.useprog();
        let mut id : GLuint = 0;
        let mut buf_id : GLuint = 0;
        let loc = unsafe {gl::GetAttribLocation(prog.id(), location.as_ptr() as *const _)};check_err!();
        let loc : GLuint = loc.try_into().map_err(|_|anyhow::Error::msg(format!("{:?} is not a valid location name", location)))?;
        unsafe { gl::GenVertexArrays(1, &mut id as *mut _);}check_err!();
        unsafe { gl::BindVertexArray(id);}check_err!();
        unsafe {gl::GenBuffers(1, &mut buf_id as *mut _);}check_err!();
        unsafe {gl::BindBuffer(gl::ARRAY_BUFFER, buf_id);}check_err!();
        unsafe {gl::BufferData(gl::ARRAY_BUFFER, (data.len() * size_of::<GLfloat>()) as isize, data.as_ptr() as *const GLvoid, gl::STATIC_DRAW);}check_err!();
        unsafe {gl::VertexAttribPointer(loc, T::size(), T::element_type(), gl::FALSE, T::stride(), null());}check_err!();
        unsafe {gl::EnableVertexAttribArray(loc);}check_err!();
        unsafe { gl::BindVertexArray(0);}check_err!();
        Ok(Self { id, _data: PhantomData::default() })
    }

    fn usevert(&self) {
        unsafe {gl::BindVertexArray(self.id);}check_err!();
    }

    fn stopusevert(&self) {
        unsafe {gl::BindVertexArray(0);}check_err!();
    }
}

pub struct DrawCall<V,U> {
    program : Program,
    vertices : Vertices<V>,
    vertexnb : usize,
    pub uniforms : U,
}

impl<V : VertexDataType,U> DrawCall<V,U> {
    pub fn new(program : Program, data : &[GLfloat], location_name : &CStr, uniforms : U) -> Result<Self> {
        let vertices = Vertices::new(&program, data, location_name)?;
        Ok(Self { program, vertices, vertexnb : data.len() /(V::size() as usize), uniforms })

    }


    pub fn update<Uv : UniformValue, F : FnOnce(&mut U) -> &mut Uniform<Uv>>(&mut self, newval : Uv, f : F) {
        self.program.useprog();
        f(&mut self.uniforms).update(newval);
    }
    
}

impl<U> DrawCall<Vec3, U> {

    pub fn draw(&self) {
        self.program.useprog();
        self.vertices.usevert();
        unsafe { gl::PatchParameteri(gl::PATCH_VERTICES, 4);}check_err!();
        unsafe { gl::DrawArrays(gl::PATCHES, 0, self.vertexnb as GLsizei);}check_err!();
        self.vertices.stopusevert();

    }
}

impl<U> DrawCall<Vec2, U> {

    pub fn draw(&self) {
        self.program.useprog();
        self.vertices.usevert();
        unsafe { gl::DrawArrays(gl::POINTS, 0, self.vertexnb as GLsizei);}check_err!();
        self.vertices.stopusevert();

    }
}

pub struct Uniform<T> {
    location : GLint,
    val : T
}

pub trait UniformValue : Sized {
    fn set(u : &Uniform<Self>);
}

impl UniformValue for GLfloat {
    fn set(u : &Uniform<Self>) {
        unsafe { gl::Uniform1f(u.location, u.val);}check_err!()
    }
}

impl UniformValue for GLint {
    fn set(u : &Uniform<Self>) {
        unsafe { gl::Uniform1i(u.location, u.val);}check_err!()
    }
}

pub struct Texture2D{
    id : GLuint,
    offset : GLenum,
}
type Rbg8Image = ImageBuffer<Rgba<u8>, Vec<u8>>;


impl Texture2D {
    pub fn new(img : &Rbg8Image, prog : &Program) -> Self {
        let offset : GLenum = 0;
        prog.useprog();
        let mut id : GLuint = 0;    
        unsafe { gl::GenTextures(1, &mut id as *mut _)}check_err!();
        unsafe { gl::BindTexture(gl::TEXTURE_2D, id)}check_err!();
        unsafe { gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, img.width() as GLsizei, img.height() as GLsizei, 0,  gl::RGBA, gl::UNSIGNED_BYTE, img.as_ptr() as *const _) }check_err!();
        unsafe { gl::GenerateMipmap(gl::TEXTURE_2D)}check_err!();
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0)}check_err!();
        Texture2D { id, offset }
    }
}

impl UniformValue for Texture2D {
    fn set(u : &Uniform<Self>) {
        println!("loc : {}, offset : {}", u.location, u.val.offset as i32);
        unsafe { gl::Uniform1i(u.location, u.val.offset as i32);}check_err!();
        unsafe { gl::ActiveTexture(gl::TEXTURE0)}check_err!();
        unsafe { gl::BindTexture(gl::TEXTURE_2D, u.val.id)}check_err!();
    }
}   


impl UniformValue for [[GLfloat;4];4] {

    fn set(u : &Uniform<Self>) {
        unsafe { gl::UniformMatrix4fv(u.location, 1, gl::FALSE, &u.val[0][0] as *const _);}check_err!()
    }
}

impl<T : UniformValue> Uniform<T> {
    pub fn new(name : &CStr, prog : &Program, val : T) -> Self {
        prog.useprog();
        let location = unsafe {
            gl::GetUniformLocation(prog.id(), name.as_ptr() as *const _)
        };check_err!();
        
        let out = Uniform { location, val };
        out.send_val();
        out
    }

    fn send_val(&self) {
        if self.location != -1 {
            T::set(self)
        }
    }

    fn update(&mut self, newval : T) {
        self.val = newval;
        self.send_val();
    }
}

