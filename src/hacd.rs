use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use rapier3d::{math::Point};
use std::collections::HashMap;

fn get_attribute(mesh: &Mesh, name: impl Into<String>) -> Option<Vec<Vec3>> {
    let name = name.into();
    let attr = mesh.attributes.iter().find(|a| a.name == name)?;
    Some(match &attr.values {
        VertexAttributeValues::Float3(v) => v.iter().map(|p| Vec3::new(p[0], p[1], p[2])).collect::<Vec<_>>(),
        _ => unreachable!()
    })
}

fn edge(a: u32, b: u32) -> (usize, usize) {
    if a > b {
        (b as usize, a as usize)
    } else {
        (a as usize, b as usize)
    }
}

pub fn hacd(mesh: &Mesh) -> Option<()> {
    let indices = mesh.indices.as_ref().unwrap()
        .chunks(3)
        .map(|c| Point::new(c[0], c[1], c[2]))
        .collect::<Vec<_>>();
    let vertices = get_attribute(mesh, "Vertex_Position")?;
    let normals = get_attribute(mesh, "Vertex_Normal")?;

    let dual_graph = {
        let mut edge_to_vertex = HashMap::new();
        for (i, tri) in indices.iter().enumerate() {
            let tri_edges = [edge(tri.x, tri.y), edge(tri.y, tri.z), edge(tri.z, tri.x)];
            for e in &tri_edges {
                edge_to_vertex.entry(*e).or_insert_with(|| vec![]).push(i);
            }
        }

        let mut dual_graph = HashMap::new();
        for adjacent_vertices in edge_to_vertex.values() {
            for v1 in adjacent_vertices.iter() {
                for v2 in adjacent_vertices.iter() {
                    if v1 != v2 {
                        dual_graph.entry(*v1).or_insert_with(|| vec![]).push(*v2);
                        dual_graph.entry(*v2).or_insert_with(|| vec![]).push(*v1);
                    }
                }
            }
        }
    };

    

    Some(())
}