# Chapter 7: Intermediate Representations — HIR → CapabilityGraph → CTL → WIT

> *"Setiap compiler yang baik mempunyai IR. Logicodex mempunyai tiga — kerana setiap peringkat kompilasi memerlukan abstraction yang berbeza."*

---

## HIR: High-Level IR (v1.36) {#hir}

### Definisi

HIR (High-Level Intermediate Representation) adalah perwakilan peringkat pertama selepas AST. Ia menormalkan AST ke dalam bentuk yang lebih terstruktur untuk analisis semantik dan lowering ke LLVM IR.

### Struktur HIR

```rust
enum HIRExpr {
    Literal(HIRLiteral),           // 42, 3.14, "hello"
    Variable(HIRVariable),         // x, counter
    Binary(HIRBinaryOp, Box<HIRExpr>, Box<HIRExpr>),  // a + b
    Call(HIRCall),                 // foo(a, b)
    If(Box<HIRExpr>, Box<HIRExpr>, Option<Box<HIRExpr>>),  // if-else
    Block(Vec<HIRExpr>),           // { stmt1; stmt2; }
    Return(Option<Box<HIRExpr>>),  // return x;
    Let(HIRVariable, Box<HIRExpr>), // let x = expr;
}

struct HIRFunction {
    name: String,
    params: Vec<(String, HIRType)>,
    return_type: HIRType,
    body: HIRExpr,
    capability_requirements: Vec<CapabilityRef>,
}

struct HIRModule {
    functions: Vec<HIRFunction>,
    services: Vec<HIRService>,
    actors: Vec<HIRActor>,
    topology: ShardTopology,
}
```

### Perbezaan HIR vs AST

| Aspek | AST | HIR |
|---|---|---|
| **Tujuan** | Struktur sintaks | Struktur semantik |
| **Alias** | Masih ada (MULA, PAPAR) | Tiada — semua kanonikal |
| **Type** | Opsional | Mandatory setiap ungkapan |
| **Capability** | Tiada | Dilampirkan pada setiap function/call |
| **Ownership** | Tiada | Dilampirkan pada setiap variable transfer |

### Lowering AST → HIR

```rust
fn lower_ast_to_hir(ast: &AST) -> Result<HIRModule, LoweringError> {
    let mut functions = Vec::new();
    let mut services = Vec::new();
    let mut actors = Vec::new();
    
    for decl in &ast.declarations {
        match decl {
            ASTDecl::Function(f) => functions.push(lower_function(f)?),
            ASTDecl::Service(s)  => services.push(lower_service(s)?),
            ASTDecl::Actor(a)    => actors.push(lower_actor(a)?),
        }
    }
    
    let topology = extract_topology(&services, &actors)?;
    
    Ok(HIRModule { functions, services, actors, topology })
}
```

### Analisis Capability pada HIR

Selepas lowering, analisis capability dijalankan pada HIR:

```rust
fn analyze_capabilities(hir: &HIRModule) -> Result<CapabilityGraph, AnalysisError> {
    let mut graph = CapabilityGraph::new();
    
    for func in &hir.functions {
        for call in func.calls() {
            let required = infer_capability(call)?;
            if !has_permission(&func, &required) {
                return Err(MissingCapability {
                    function: func.name.clone(),
                    call: call.name.clone(),
                    required,
                });
            }
        }
    }
    
    Ok(graph)
}
```

---

## CapabilityGraph IR: Single Source of Truth (v1.35) {#capgraph}

### Definisi

CapabilityGraph IR adalah "Single Source of Truth" yang menyatukan tiga struktur yang sebelum ini berasingan: SemanticSummary (v1.31), CapabilityTopology (v1.32), dan ShardTopology (v1.34).

### Struktur Graph

```rust
struct CapabilityGraph {
    services: Vec<IRServiceNode>,
    gates: Vec<IRGateEdge>,
    shards: Vec<IRShardNode>,
    doors: Vec<IRDoorEdge>,
    target: CompileTarget,
}

struct IRServiceNode {
    id: ServiceId,
    name: String,
    port: Option<u16>,
    handler: String,
    effects: Vec<SideEffect>,
    inline_cost: usize,
    assigned_shard: Option<ShardId>,
}

struct IRGateEdge {
    from: ServiceId,
    to: ServiceId,
    domain: CapabilityDomain,
    operation: CapabilityOperation,
    capability_ref: CapabilityRef,
    gate_type: GateType,  // DirectCall | Message | Hardware
}

struct IRShardNode {
    id: ShardId,
    core_id: usize,
    memory_budget: usize,
    assigned_services: Vec<ServiceId>,
}

struct IRDoorEdge {
    from_shard: ShardId,
    to_shard: ShardId,
    channel_type: ChannelType,
    capacity: usize,
}
```

### Pembinaan Graph

```rust
impl CapabilityGraph {
    fn from_hir(hir: &HIRModule) -> Result<Self, GraphError> {
        let mut graph = CapabilityGraph::new(hir.target);
        
        // 1. Tambah service nodes
        for service in &hir.services {
            graph.add_service(IRServiceNode {
                id: ServiceId::new(),
                name: service.name.clone(),
                port: service.port,
                handler: service.handler.clone(),
                effects: analyze_effects(service)?,
                inline_cost: estimate_inline_cost(service)?,
                assigned_shard: None,  // akan diisi oleh sharding algorithm
            });
        }
        
        // 2. Tambah gate edges dari capability requirements
        for service in &hir.services {
            for req in &service.capability_requirements {
                graph.add_gate(IRGateEdge {
                    from: service.id,
                    to: req.target_service,
                    domain: req.domain,
                    operation: req.operation,
                    capability_ref: req.capability_ref.clone(),
                    gate_type: infer_gate_type(req)?,
                });
            }
        }
        
        // 3. Tambah shard nodes dan assign services
        for (core_id, shard_services) in sharding_algorithm(&graph)? {
            let shard_id = graph.add_shard(IRShardNode {
                id: ShardId::new(),
                core_id,
                memory_budget: calculate_budget(&shard_services)?,
                assigned_services: shard_services.iter().map(|s| s.id).collect(),
            });
            
            for service in &shard_services {
                graph.assign_service_to_shard(service.id, shard_id)?;
            }
        }
        
        // 4. Tambah door edges untuk cross-shard communication
        for gate in &graph.gates {
            let from_shard = graph.shard_for(gate.from)?;
            let to_shard = graph.shard_for(gate.to)?;
            if from_shard != to_shard {
                graph.add_door(IRDoorEdge {
                    from_shard,
                    to_shard,
                    channel_type: ChannelType::from_gate_type(&gate.gate_type),
                    capacity: DEFAULT_CHANNEL_CAPACITY,
                });
            }
        }
        
        Ok(graph)
    }
}
```

### 6 Semakan Verifikasi

```rust
impl CapabilityGraph {
    fn verify(&self) -> Result<(), VerificationError> {
        // 1. Graph tidak kosong
        if self.services.is_empty() {
            return Err(EmptyGraph);
        }
        
        // 2. WASM target tidak ada hardware gate
        if self.target == CompileTarget::Wasm {
            for gate in &self.gates {
                if gate.gate_type == GateType::Hardware {
                    return Err(WasmHardwareGate(gate.domain.clone()));
                }
            }
        }
        
        // 3. Tiada shard assignment tidak sah
        for service in &self.services {
            if let Some(shard_id) = service.assigned_shard {
                if !self.shards.iter().any(|s| s.id == shard_id) {
                    return Err(InvalidShardAssignment(service.id, shard_id));
                }
            }
        }
        
        // 4. Semua door merujuk service yang wujud
        for door in &self.doors {
            if !self.shards.iter().any(|s| s.id == door.from_shard) {
                return Err(UnknownServiceInDoor(door.from_shard));
            }
            if !self.shards.iter().any(|s| s.id == door.to_shard) {
                return Err(UnknownServiceInDoor(door.to_shard));
            }
        }
        
        // 5. Semua gate merujuk service yang wujud
        for gate in &self.gates {
            if !self.services.iter().any(|s| s.id == gate.from) {
                return Err(UnknownServiceInGate(gate.from));
            }
            if !self.services.iter().any(|s| s.id == gate.to) {
                return Err(UnknownServiceInGate(gate.to));
            }
        }
        
        // 6. Tiada shard kosong
        for shard in &self.shards {
            if shard.assigned_services.is_empty() {
                return Err(EmptyShard(shard.id));
            }
        }
        
        Ok(())
    }
}
```

---

## CTL Mapper: Capability Translation Layer (v1.36) {#ctl}

### Definisi

CTL Mapper memetakan model capability-native Logicodex ke dalam ekosistem WASM. Prinsipnya: **"Project INTO, not borrow FROM."**

### Pemetaan Domain ke WIT

| Domain Logicodex | WIT Target | Hardware? | Keterangan |
|---|---|---|---|
| `Storage` | `wasi:filesystem` | Tidak | Fail system melalui WASI |
| `Net` | `wasi:sockets` | Tidak | Rangkaian melalui WASI |
| `UI` | `wasi:cli` | Tidak | CLI/environment melalui WASI |
| `HW` | `logicodex:host-reactor` | **Ya — mediated** | Hardware melalui Host Reactor |
| `Audio` | `wasi:io/custom` | Tidak | Audio melalui WASI custom |
| `Crypto` | `wasi:crypto` | Tidak | Kripto melalui WASI |

### Algoritma Pemetaan

```rust
struct CtlMapper {
    overrides: HashMap<String, String>,  // manual overrides
}

impl CtlMapper {
    fn new() -> Self {
        let mut overrides = HashMap::new();
        CtlMapper { overrides }
    }
    
    fn add_override(&mut self, logicodex_op: &str, wit_binding: &str) {
        self.overrides.insert(logicodex_op.to_string(), wit_binding.to_string());
    }
    
    fn map_to_wit(&self, gate: &IRGateEdge) -> String {
        // 1. Periksa override manual
        let key = format!("{}.{}", gate.domain, gate.operation);
        if let Some(override_val) = self.overrides.get(&key) {
            return override_val.clone();
        }
        
        // 2. Pemetaan automatik
        match gate.domain {
            CapabilityDomain::Storage => format!("wasi:filesystem/{}", gate.operation),
            CapabilityDomain::Net     => format!("wasi:sockets/{}", gate.operation),
            CapabilityDomain::UI      => format!("wasi:cli/{}", gate.operation),
            CapabilityDomain::HW      => {
                // Hardware gate TIDAK PERNAH sampai ke guest WASM
                // Sentiasa melalui Host Reactor
                format!("logicodex:host-reactor/{}", gate.operation)
            }
            CapabilityDomain::Audio   => format!("wasi:io/custom/{}", gate.operation),
            CapabilityDomain::Crypto  => format!("wasi:crypto/{}", gate.operation),
            _ => format!("logicodex:custom/{}", gate.operation),  // fallback
        }
    }
    
    fn generate_wit(&self, graph: &CapabilityGraph) -> String {
        let mut wit = String::new();
        wit.push_str("package logicodex:generated;\n\n");
        
        for service in &graph.services {
            wit.push_str(&format!("interface {} {{\n", service.name));
            
            // Tambah handler function
            wit.push_str(&format!("  {}: func(req: request) -> response;\n", service.handler));
            
            // Tambah capability imports
            for gate in graph.gates_for(service.id) {
                let wit_import = self.map_to_wit(gate);
                wit.push_str(&format!("  use {};\n", wit_import));
            }
            
            wit.push_str("}\n\n");
        }
        
        // Tambah world definition
        wit.push_str("world logicodex-world {\n");
        for service in &graph.services {
            wit.push_str(&format!("  import {};\n", service.name));
        }
        wit.push_str("}\n");
        
        wit
    }
}
```

### Output: Host Reactor Stubs

Untuk gate hardware, CTL Mapper menjana stub Rust untuk Host Reactor:

```rust
// Generated by CTL Mapper for HW.GPIO

#[derive(Debug)]
enum HostFunction {
    GpioControl { pin: u8, value: bool },
    TimerSet { id: u8, duration_ms: u64 },
    DmaTransfer { src: u64, dst: u64, len: usize },
}

struct HostReactor {
    permissions: GatePermissions,
    hardware_zones: HashMap<u8, HardwareZone>,
}

impl HostReactor {
    fn dispatch(&mut self, request: GuestRequest) -> HostResponse {
        match request.function {
            HostFunction::GpioControl { pin, value } => {
                // Periksa permission
                if !self.permissions.allows("HW.GPIO", pin) {
                    return HostResponse::PermissionDenied;
                }
                
                // Periksa hardware zone
                let zone = self.hardware_zones.entry(pin).or_default();
                if zone.is_claimed && zone.owner != request.guest_id {
                    return HostResponse::HardwareBusy;
                }
                
                // Eksekusi
                gpio_set(pin, value);
                HostResponse::Ok
            }
            // ... other functions
        }
    }
}
```

---

## Output: Native ELF / `.cap` / WIT dari Satu IR {#output}

### Pipeline Output

```text
CapabilityGraph IR
        │
        ├──► Native Codegen ──► ELF executable (x86_64/aarch64/riscv64)
        │
        ├──► .cap Generator ──► topology.cap (audit trail)
        │
        └──► CTL Mapper ──► WIT file ──► WASM module
```

### Native Codegen

```rust
fn generate_native(graph: &CapabilityGraph) -> Result<Vec<u8>, CodegenError> {
    let mut codegen = NativeCodegen::new(graph.target.arch()?);
    
    for service in &graph.services {
        // 1. Hasilkan function wrapper dengan inlined gate checks
        codegen.emit_service_function(service)?;
        
        // 2. Hasilkan capability check (compiled-time → zero runtime cost)
        for gate in graph.gates_for(service.id) {
            codegen.emit_capability_check(gate)?;  // inlined, no-op at runtime
        }
        
        // 3. Hasilkan shard assignment
        if let Some(shard_id) = service.assigned_shard {
            let shard = graph.shard(shard_id)?;
            codegen.emit_affinity_hint(shard.core_id)?;
        }
    }
    
    // 4. Hasilkan door (cross-shard channel setup)
    for door in &graph.doors {
        codegen.emit_door_setup(door)?;
    }
    
    Ok(codegen.finish()?)
}
```

### `.cap` Generator

```rust
fn generate_cap(graph: &CapabilityGraph) -> String {
    let mut cap = String::new();
    cap.push_str("; Logicodex Capability Audit Trail\n");
    cap.push_str(&format!("; Generated: {}\n", Utc::now().to_rfc3339()));
    cap.push_str(&format!("; Target: {:?}\n", graph.target));
    cap.push_str("; =================================\n\n");
    
    for service in &graph.services {
        cap.push_str(&format!("[service {}]\n", service.name));
        if let Some(port) = service.port {
            cap.push_str(&format!("port={}\n", port));
        }
        cap.push_str(&format!("handler={}\n", service.handler));
        
        for gate in graph.gates_for(service.id) {
            cap.push_str(&format!("\n[gate {}.{}]\n", gate.domain, gate.operation));
            cap.push_str(&format!("type={:?}\n", gate.gate_type));
            cap.push_str(&format!("domain={:?}\n", gate.domain));
            cap.push_str(&format!("operation={}\n", gate.operation));
            cap.push_str(&format!("verified=true\n"));
            
            // Checksum untuk integriti
            let checksum = compute_checksum(&gate);
            cap.push_str(&format!("checksum=sha256:{}\n", checksum));
        }
        
        cap.push_str("\n");
    }
    
    cap
}
```

### Kebaikan "Single Source of Truth"

| Aspek | Sebelum v1.35 (3 struktur) | Selepas v1.35 (1 IR) |
|---|---|---|
| Konsistensi | Mungkin tidak konsisten | Dijamin — satu sumber |
| Verifikasi | 3 set semakan | 1 set semakan (6 checks) |
| Maintenance | 3 struktur untuk dikemas kini | 1 struktur |
| Output | Mungkin tidak selaras | Sentiasa selaras (Native/`.cap`/WIT) |
| Debugging | Sukar trace | Mudah — satu graph untuk diperiksa |

---

## Ringkasan IR Pipeline

```text
AST ──► HIR ──► CapabilityGraph ──► [Native | .cap | WIT]
        │             │
        │             ├── 6 verify() checks
        │             │
        │             └── CTL Mapper (WASM)
        │
        └── Type check
            Capability analysis
            Ownership tracking
```

| IR | Fungsi | Fail |
|---|---|---|
| **AST** | Struktur sintaks | `src/parser.rs` |
| **HIR** | Struktur semantik | `src/hir.rs` |
| **CapabilityGraph** | Single Source of Truth | `src/tier2/capability_ir.rs` |
| **CTL Mapper** | WIT generation | `src/tier2/ctl_mapper.rs` |
| **WIT** | WASM interface types | Output |
| **Native ELF** | Executable binari | Output |
| **`.cap`** | Audit trail | Output |

Setiap peringkat IR menambah maklumat (type, capability, topology) dan membuang detail yang tidak relevan (alias, lokasi sumber). Ini adalah pattern compiler standard — tetapi dalam Logicodex, setiap peringkat juga menambah jaminan keselamatan.
