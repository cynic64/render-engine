use crate::internal_tools::*;
use crate::exposed_tools::*;
use crate::creator::VbufCreator;

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

pub struct Object {
    vbuf: Arc<VertexBuffer>,
}

pub struct World {
    objects: HashMap<String, Object>,
    vbuf_creator: VbufCreator,
    // we need to use an option to get around the borrow checker later
    // soooooorry
    command_recv: Option<Receiver<Command>>,
    // we store a copy of the sender as well so we can clone it and give it
    // out to whoever needss it
    command_send: Sender<Command>,
}

pub struct WorldCommunicator {
    command_send: Sender<Command>,
}

pub enum Command {
    ObjectFromVbuf {
        id: String,
        vbuf: Arc<VertexBuffer>,
    },
    ObjectFromVerts {
        id: String,
        verts: Vec<Vertex>,
    },
    DeleteObject {
        id: String,
    },
}

impl World {
    pub fn from_creator(vbuf_creator: VbufCreator) -> Self {
        let (sender, receiver): (Sender<Command>, Receiver<Command>) = mpsc::channel();

        Self {
            objects: HashMap::new(),
            vbuf_creator,
            command_recv: Some(receiver),
            command_send: sender,
        }
    }

    pub fn get_communicator(&self) -> WorldCommunicator {
        WorldCommunicator::from_sender(self.command_send.clone())
    }

    pub fn add_object_from_vbuf(&mut self, id: String, vbuf: Arc<VertexBuffer>) {
        let new_object = Object {
            vbuf,
        };

        self.objects.insert(id, new_object);
    }

    pub fn add_object_from_verts(&mut self, id: String, verts: Vec<Vertex>) {
        let vbuf = self.vbuf_creator.create_vbuf_from_verts(&verts);
        let new_object = Object {
            vbuf,
        };

        self.objects.insert(id, new_object);
    }

    pub fn get_vbufs(&self) -> Vec<Arc<VertexBuffer>> {
        self.objects.values().map(|object| object.vbuf.clone()).collect()
    }

    pub fn delete_object(&mut self, id: String) {
        self.objects.remove(&id);
    }

    pub fn check_for_commands(&mut self) {
        let command_recv = self.command_recv.take().unwrap();

        command_recv.try_iter().for_each(|command| match command {
            Command::ObjectFromVbuf {id, vbuf} => self.add_object_from_vbuf(id, vbuf),
            Command::ObjectFromVerts {id, verts} => self.add_object_from_verts(id, verts),
            Command::DeleteObject {id} => self.delete_object(id),
        });

        self.command_recv = Some(command_recv);
    }
}

impl WorldCommunicator {
    pub fn from_sender(sender: Sender<Command>) -> Self {
        Self {
            command_send: sender,
        }
    }

    pub fn add_object_from_vbuf(&mut self, id: String, vbuf: Arc<VertexBuffer>) {
        let command = Command::ObjectFromVbuf{
            id,
            vbuf,
        };

        self.command_send.send(command).unwrap();
    }

    pub fn add_object_from_verts(&mut self, id: String, verts: Vec<Vertex>) {
        let command = Command::ObjectFromVerts {
            id,
            verts,
        };

        self.command_send.send(command).unwrap();
    }

    pub fn delete_object(&mut self, id: String) {
        let command = Command::DeleteObject {
            id,
        };

        self.command_send.send(command).unwrap();
    }
}
