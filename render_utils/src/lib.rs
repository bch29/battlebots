extern crate glium;

#[derive(Debug, Clone)]
pub struct DataBuilder<'s, V, U>
    where V: glium::Vertex + Copy,
          U: IntoUniforms {

    pub vertices: Vec<V>,
    pub indices: Vec<u16>,
    pub uniforms: U,
    pub vshader_src: &'s str,
    pub fshader_src: &'s str,
}

impl<'s, V, U> DataBuilder<'s, V, U>
    where V: glium::Vertex + Copy,
          U: IntoUniforms {

    pub fn build_instanced<F, I>(
        self,
        display: &F,
        max_instances: usize,
        primitive_type: glium::index::PrimitiveType)
        -> Result<InstancedData<V, I, U::IntoUniforms>, Error>

        where F: glium::backend::Facade,
              I: glium::Vertex + Copy {

        Ok(InstancedData {
            vertex_buf: try!(glium::VertexBuffer::new(display, &self.vertices)),
            index_buf: try!(glium::IndexBuffer::new(display, primitive_type, &self.indices)),
            instance_buf: try!(glium::VertexBuffer::empty_persistent(display, max_instances)),
            uniforms: self.uniforms.into_uniforms(),
            program: try!(glium::Program::from_source(display, self.vshader_src, self.fshader_src, None)),
        })
    }
}

pub struct InstancedData<V, I, U>
    where V: Copy,
          I: Copy,
          U: glium::uniforms::Uniforms {

    pub vertex_buf: glium::VertexBuffer<V>,
    pub index_buf: glium::IndexBuffer<u16>,
    pub instance_buf: glium::VertexBuffer<I>,
    pub uniforms: U,
    pub program: glium::Program,
}

impl<V, I, U> InstancedData<V, I, U>
    where V: Copy,
          I: Copy,
          U: glium::uniforms::Uniforms {

    pub fn update_instances<Instances, Iter>(&mut self, instances: Instances) -> Result<(), MaxInstancesExceeded>
        where Instances: IntoIterator<Item = I, IntoIter = Iter>, Iter: ExactSizeIterator<Item = I>
    {
        let iter = instances.into_iter();

        if self.instance_buf.len() < iter.len() {
            return Err(MaxInstancesExceeded);
        }

        for (dest, src) in self.instance_buf.map().iter_mut().zip(iter) {
            *dest = src;
        }

        Ok(())
    }

    pub fn draw<S: glium::Surface>(
        &self,
        surface: &mut S,
        params: &glium::DrawParameters)
        -> Result<(), glium::DrawError> {

        surface.draw((&self.vertex_buf, self.instance_buf.per_instance().unwrap()), &self.index_buf, &self.program, &self.uniforms, params)
    }
}

pub trait ToVertex {
    type ToVertex: glium::Vertex + Copy;

    fn to_vertex(&self) -> Self::ToVertex;
}

pub trait IntoUniforms {
    type IntoUniforms: glium::uniforms::Uniforms;

    fn into_uniforms(self) -> Self::IntoUniforms;
}

impl IntoUniforms for () {
    type IntoUniforms = glium::uniforms::EmptyUniforms;

    fn into_uniforms(self) -> Self::IntoUniforms {
        glium::uniforms::EmptyUniforms
    }
}

impl IntoUniforms for glium::uniforms::EmptyUniforms {
    type IntoUniforms = Self;

    fn into_uniforms(self) -> Self { self }
}

impl<'n, T, R> IntoUniforms for glium::uniforms::UniformsStorage<'n, T, R>
    where T: glium::uniforms::AsUniformValue,
          R: glium::uniforms::Uniforms {

    type IntoUniforms = Self;

    fn into_uniforms(self) -> Self { self }
}

#[derive(Debug, Clone)]
pub enum Error {
    VertexBufferCreationError(glium::vertex::BufferCreationError),
    IndexBufferCreationError(glium::index::BufferCreationError),
    ProgramCreationError(glium::program::ProgramCreationError),
}

#[derive(Debug, Clone, Copy)]
pub struct MaxInstancesExceeded;

impl From<glium::vertex::BufferCreationError> for Error {
    fn from(err: glium::vertex::BufferCreationError) -> Error {
        Error::VertexBufferCreationError(err)
    }
}

impl From<glium::index::BufferCreationError> for Error {
    fn from(err: glium::index::BufferCreationError) -> Error {
        Error::IndexBufferCreationError(err)
    }
}

impl From<glium::program::ProgramCreationError> for Error {
    fn from(err: glium::program::ProgramCreationError) -> Error {
        Error::ProgramCreationError(err)
    }
}
