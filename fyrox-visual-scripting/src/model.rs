use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GraphId(pub String);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphKind {
    Event,
    Construction,
    Function,
    Graph,
}

impl Default for GraphKind {
    fn default() -> Self {
        Self::Graph
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDef {
    pub name: String,
    #[serde(default)]
    pub kind: GraphKind,
}

fn default_graphs() -> Vec<GraphDef> {
    vec![
        GraphDef {
            name: "EventGraph".to_string(),
            kind: GraphKind::Event,
        },
        GraphDef {
            name: "ConstructionScript".to_string(),
            kind: GraphKind::Construction,
        },
    ]
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PinId(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinDirection {
    Input,
    Output,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Exec,
    Bool,
    I32,
    F32,
    String,
    Unit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    I32(i32),
    F32(f32),
    String(String),
    Unit,
}

impl Value {
    pub fn data_type(&self) -> DataType {
        match self {
            Value::Bool(_) => DataType::Bool,
            Value::I32(_) => DataType::I32,
            Value::F32(_) => DataType::F32,
            Value::String(_) => DataType::String,
            Value::Unit => DataType::Unit,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuiltinNodeKind {
    BeginPlay,
    Tick,
    ConstructionScript,
    Print,
    RhaiScript,
    Branch,
    GetVariable,
    SetVariable,
    Self_,
    GetActorTransform,
    SetActorTransform,
    SpawnActor,
    GetActorByName,
    GetActorName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    pub id: PinId,
    pub name: String,
    pub direction: PinDirection,
    pub data_type: DataType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: BuiltinNodeKind,
    #[serde(default = "default_node_graph")]
    pub graph: String,
    #[serde(default)]
    pub position: [f32; 2],
    pub pins: Vec<Pin>,
    pub properties: BTreeMap<String, Value>,
}

impl Node {
    pub fn new(kind: BuiltinNodeKind) -> Self {
        // IDs are assigned by the graph.
        let mut node = Self {
            id: NodeId(0),
            kind,
            graph: default_node_graph(),
            position: [0.0, 0.0],
            pins: Vec::new(),
            properties: BTreeMap::new(),
        };

        node.pins = default_pins(kind);
        const DEFAULT_RHAI_CODE: &str = "// Rhai snippet examples\n//\n// 1) Log\n// print(\"Hello from Rhai\");\n//\n// 2) Use variables\n// set_var(\"message\", \"Hello\");\n// print(get_var(\"message\"));\n//\n// 3) Read delta time during Tick\n// print(\"dt = \" + dt().to_string());\n";
        if kind == BuiltinNodeKind::RhaiScript {
            node.properties
                .insert("code".to_string(), Value::String(DEFAULT_RHAI_CODE.to_string()));
        }
        node
    }

    pub fn pin_named(&self, name: &str) -> Option<PinId> {
        self.pins.iter().find(|p| p.name == name).map(|p| p.id)
    }

    pub fn set_property_string(&mut self, key: &str, value: String) {
        self.properties.insert(key.to_string(), Value::String(value));
    }

    pub fn set_property_bool(&mut self, key: &str, value: bool) {
        self.properties.insert(key.to_string(), Value::Bool(value));
    }

    pub fn set_property_i32(&mut self, key: &str, value: i32) {
        self.properties.insert(key.to_string(), Value::I32(value));
    }

    pub fn set_property_f32(&mut self, key: &str, value: f32) {
        self.properties.insert(key.to_string(), Value::F32(value));
    }
}

fn default_node_graph() -> String {
    "EventGraph".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDef {
    pub name: String,
    #[serde(default = "default_variable_type")]
    pub data_type: DataType,
    #[serde(default)]
    pub default_value: Option<Value>,
}

fn default_variable_type() -> DataType {
    DataType::String
}

fn default_pins(kind: BuiltinNodeKind) -> Vec<Pin> {
    use BuiltinNodeKind as K;
    use DataType as T;
    use PinDirection as D;

    match kind {
        K::BeginPlay | K::ConstructionScript => vec![
            Pin {
                id: PinId(0),
                name: "then".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
        ],
        K::Tick => vec![
            Pin {
                id: PinId(0),
                name: "then".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(1),
                name: "dt".to_string(),
                direction: D::Output,
                data_type: T::F32,
            },
        ],
        K::Print => vec![
            Pin {
                id: PinId(0),
                name: "exec".to_string(),
                direction: D::Input,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(1),
                name: "then".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(2),
                name: "text".to_string(),
                direction: D::Input,
                data_type: T::String,
            },
        ],
        K::RhaiScript => vec![
            Pin {
                id: PinId(0),
                name: "exec".to_string(),
                direction: D::Input,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(1),
                name: "then".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(2),
                name: "code".to_string(),
                direction: D::Input,
                data_type: T::String,
            },
        ],
        K::Branch => vec![
            Pin {
                id: PinId(0),
                name: "exec".to_string(),
                direction: D::Input,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(1),
                name: "condition".to_string(),
                direction: D::Input,
                data_type: T::Bool,
            },
            Pin {
                id: PinId(2),
                name: "true".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(3),
                name: "false".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
        ],
        K::GetVariable => vec![
            Pin {
                id: PinId(0),
                name: "value".to_string(),
                direction: D::Output,
                data_type: T::String,
            },
        ],
        K::SetVariable => vec![
            Pin {
                id: PinId(0),
                name: "exec".to_string(),
                direction: D::Input,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(1),
                name: "then".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(2),
                name: "value".to_string(),
                direction: D::Input,
                data_type: T::String,
            },
        ],
        K::Self_ => vec![
            Pin {
                id: PinId(0),
                name: "self".to_string(),
                direction: D::Output,
                data_type: T::String,
            },
        ],
        K::GetActorTransform => vec![
            Pin {
                id: PinId(0),
                name: "exec".to_string(),
                direction: D::Input,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(1),
                name: "then".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(2),
                name: "actor".to_string(),
                direction: D::Input,
                data_type: T::String,
            },
            Pin {
                id: PinId(3),
                name: "position".to_string(),
                direction: D::Output,
                data_type: T::String,
            },
        ],
        K::SetActorTransform => vec![
            Pin {
                id: PinId(0),
                name: "exec".to_string(),
                direction: D::Input,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(1),
                name: "then".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(2),
                name: "actor".to_string(),
                direction: D::Input,
                data_type: T::String,
            },
            Pin {
                id: PinId(3),
                name: "position".to_string(),
                direction: D::Input,
                data_type: T::String,
            },
        ],
        K::SpawnActor => vec![
            Pin {
                id: PinId(0),
                name: "exec".to_string(),
                direction: D::Input,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(1),
                name: "then".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(2),
                name: "class".to_string(),
                direction: D::Input,
                data_type: T::String,
            },
            Pin {
                id: PinId(3),
                name: "actor".to_string(),
                direction: D::Output,
                data_type: T::String,
            },
        ],
        K::GetActorByName => vec![
            Pin {
                id: PinId(1),
                name: "exec".to_string(),
                direction: D::Input,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(2),
                name: "then".to_string(),
                direction: D::Output,
                data_type: T::Exec,
            },
            Pin {
                id: PinId(3),
                name: "name".to_string(),
                direction: D::Input,
                data_type: T::String,
            },
            Pin {
                id: PinId(4),
                name: "actor".to_string(),
                direction: D::Output,
                data_type: T::String,
            },
        ],
        K::GetActorName => vec![
            Pin {
                id: PinId(0),
                name: "actor".to_string(),
                direction: D::Input,
                data_type: T::String,
            },
            Pin {
                id: PinId(1),
                name: "name".to_string(),
                direction: D::Output,
                data_type: T::String,
            },
        ],
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub from: PinId,
    pub to: PinId,
}

impl Link {
    pub fn exec(from: PinId, to: PinId) -> Self {
        Self { from, to }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintGraph {
    pub id: GraphId,
    #[serde(default = "default_graphs")]
    pub graphs: Vec<GraphDef>,
    pub nodes: BTreeMap<NodeId, Node>,
    pub links: Vec<Link>,
    #[serde(default)]
    pub variables: Vec<VariableDef>,

    next_node_id: u32,
    next_pin_id: u32,
}

impl BlueprintGraph {
    pub fn new(id: GraphId) -> Self {
        Self {
            id,
            graphs: default_graphs(),
            nodes: BTreeMap::new(),
            links: Vec::new(),
            variables: Vec::new(),
            next_node_id: 1,
            next_pin_id: 1,
        }
    }

    pub fn ensure_builtin_graphs(&mut self) {
        let mut has_event = false;
        let mut has_construction = false;
        for g in self.graphs.iter() {
            has_event |= g.name == "EventGraph";
            has_construction |= g.name == "ConstructionScript";
        }
        if !has_event {
            self.graphs.push(GraphDef {
                name: "EventGraph".to_string(),
                kind: GraphKind::Event,
            });
        }
        if !has_construction {
            self.graphs.push(GraphDef {
                name: "ConstructionScript".to_string(),
                kind: GraphKind::Construction,
            });
        }
    }

    pub fn add_graph(&mut self, name: String, kind: GraphKind) {
        if self.graphs.iter().any(|g| g.name == name) {
            return;
        }
        self.graphs.push(GraphDef { name, kind });
    }

    pub fn add_node(&mut self, mut node: Node) -> NodeId {
        let node_id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        // Remap node + pin IDs to unique graph IDs.
        node.id = node_id;
        for pin in node.pins.iter_mut() {
            pin.id = PinId(self.next_pin_id);
            self.next_pin_id += 1;
        }

        self.nodes.insert(node_id, node);
        node_id
    }

    pub fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    pub fn pin_owner(&self, pin_id: PinId) -> Option<NodeId> {
        self.nodes
            .iter()
            .find_map(|(node_id, node)| node.pins.iter().any(|p| p.id == pin_id).then_some(*node_id))
    }

    pub fn pin(&self, pin_id: PinId) -> Option<&Pin> {
        self.nodes
            .values()
            .flat_map(|n| n.pins.iter())
            .find(|p| p.id == pin_id)
    }
}
