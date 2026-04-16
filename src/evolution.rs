use crate::config;
use crate::genome::{Genome, Instruction, OpCode, Node};
use crate::genome::op::{op_def, Arity, OP_REGISTRY};
use rand::Rng;

// ============================================================================
// EVOLUTION PARAMETERS
// ============================================================================

pub const DEFAULT_SUBTREE_MUTATION_PROB: f64 = config::SUBTREE_MUTATION_PROB;
pub const DEFAULT_SUBTREE_STOP_PROB: f64 = config::SUBTREE_STOP_PROB;
pub const DEFAULT_BINARY_CHILD_SIDE_PROB: f64 = config::BINARY_CHILD_SIDE_PROB;
pub const DEFAULT_FRESH_RANDOM_COUNT: usize = config::FRESH_RANDOM_COUNT;
pub const DEFAULT_MAX_TREE_DEPTH: usize = config::MAX_TREE_DEPTH;
pub const DEFAULT_EXPRESSION_MUTATION_PROB: f64 = config::EXPRESSION_MUTATION_PROB;
pub const DEFAULT_DROPOUT_PROB: f64 = config::DROPOUT_PROB;
pub const DEFAULT_DUPLICATION_PROB: f64 = config::DUPLICATION_PROB;

/// Runtime evolution parameters (can be modified during execution)
#[derive(Clone, Copy)]
pub struct EvolutionParams {
    pub subtree_mutation_prob: f64,
    pub subtree_stop_prob: f64,
    pub binary_child_side_prob: f64,
    pub expression_mutation_prob: f64,
    pub dropout_prob: f64,
    pub duplication_prob: f64,
}

impl Default for EvolutionParams {
    fn default() -> Self {
        Self {
            subtree_mutation_prob: DEFAULT_SUBTREE_MUTATION_PROB,
            subtree_stop_prob: DEFAULT_SUBTREE_STOP_PROB,
            binary_child_side_prob: DEFAULT_BINARY_CHILD_SIDE_PROB,
            expression_mutation_prob: DEFAULT_EXPRESSION_MUTATION_PROB,
            dropout_prob: DEFAULT_DROPOUT_PROB,
            duplication_prob: DEFAULT_DUPLICATION_PROB,
        }
    }
}

pub fn mutate(genome: &Genome, rng: &mut impl Rng) -> Genome {
    mutate_with_params(genome, rng, &EvolutionParams::default())
}

pub fn mutate_with_params(genome: &Genome, rng: &mut impl Rng, params: &EvolutionParams) -> Genome {
    let mut tree = genome.tree();
    if rng.gen_bool(params.subtree_mutation_prob) {
        tree = mutate_subtree_with_params(&tree, rng, params);
    } else {
        tree = replace_node(&tree, rng);
    }
    if params.expression_mutation_prob > 0.0 {
        expression_mutate(&mut tree, params.expression_mutation_prob, rng);
    }
    if params.dropout_prob > 0.0 && rng.gen_bool(params.dropout_prob) {
        dropout_node(&mut tree, rng);
    }
    if params.duplication_prob > 0.0 && rng.gen_bool(params.duplication_prob) {
        duplicate_subtree(&mut tree, rng);
    }
    Genome::new(tree)
}

fn mutate_subtree_with_params(node: &Node, rng: &mut impl Rng, params: &EvolutionParams) -> Node {
    if rng.gen_bool(params.subtree_stop_prob) {
        return node.clone();
    }

    let def = op_def(node.op);
    let mut result = node.clone();

    match def.arity {
        Arity::Nullary => {
            if node.op == OpCode::Const {
                result.value = rng.gen::<f32>();
            }
        }
        Arity::Unary => {
            result.children[0] = mutate_subtree_with_params(&node.children[0], rng, params);
        }
        Arity::Binary => {
            // FBM: 1/3 chance to mutate octaves instead of a child
            if node.op == OpCode::FBM {
                match rng.gen_range(0..3) {
                    0 => result.children[0] = mutate_subtree_with_params(&node.children[0], rng, params),
                    1 => result.children[1] = mutate_subtree_with_params(&node.children[1], rng, params),
                    _ => result.c_literal = rng.gen_range(1..=8),
                }
            } else if rng.gen_bool(params.binary_child_side_prob) {
                result.children[0] = mutate_subtree_with_params(&node.children[0], rng, params);
            } else {
                result.children[1] = mutate_subtree_with_params(&node.children[1], rng, params);
            }
        }
        Arity::Ternary => {
            // Mix: only mutate first two children (preserve blend parameter t)
            if node.op == OpCode::Mix {
                if rng.gen_bool(params.binary_child_side_prob) {
                    result.children[0] = mutate_subtree_with_params(&node.children[0], rng, params);
                } else {
                    result.children[1] = mutate_subtree_with_params(&node.children[1], rng, params);
                }
            } else {
                let idx = rng.gen_range(0..3);
                result.children[idx] = mutate_subtree_with_params(&node.children[idx], rng, params);
            }
        }
    }

    result
}

fn replace_node(node: &Node, rng: &mut impl Rng) -> Node {
    let def = op_def(node.op);

    match def.arity {
        Arity::Nullary => {
            if matches!(node.op, OpCode::X | OpCode::Y) {
                node.clone()
            } else {
                Node::random(rng)
            }
        }
        Arity::Unary => {
            let mut result = node.clone();
            result.children[0] = replace_node(&node.children[0], rng);
            result
        }
        Arity::Binary => {
            // 2/3 recurse into a child, 1/3 replace whole node
            match rng.gen_range(0..3) {
                0 => {
                    let mut result = node.clone();
                    result.children[0] = replace_node(&node.children[0], rng);
                    result
                }
                1 => {
                    let mut result = node.clone();
                    result.children[1] = replace_node(&node.children[1], rng);
                    result
                }
                _ => Node::random(rng),
            }
        }
        Arity::Ternary => {
            // Mix: only replace first two children or whole
            if node.op == OpCode::Mix {
                match rng.gen_range(0..3) {
                    0 => {
                        let mut result = node.clone();
                        result.children[0] = replace_node(&node.children[0], rng);
                        result
                    }
                    1 => {
                        let mut result = node.clone();
                        result.children[1] = replace_node(&node.children[1], rng);
                        result
                    }
                    _ => Node::random(rng),
                }
            } else {
                // 3/4 recurse into a child, 1/4 replace whole node
                match rng.gen_range(0..4) {
                    0 => {
                        let mut result = node.clone();
                        result.children[0] = replace_node(&node.children[0], rng);
                        result
                    }
                    1 => {
                        let mut result = node.clone();
                        result.children[1] = replace_node(&node.children[1], rng);
                        result
                    }
                    2 => {
                        let mut result = node.clone();
                        result.children[2] = replace_node(&node.children[2], rng);
                        result
                    }
                    _ => Node::random(rng),
                }
            }
        }
    }
}

// ---- Palette genome mutation (PaletteT is the "X/Y" of palette trees) ----

pub fn mutate_palette_with_params(genome: &Genome, rng: &mut impl Rng, params: &EvolutionParams) -> Genome {
    let mut candidate = genome.clone();
    for _ in 0..10 {
        let mut tree = genome.tree();
        if rng.gen_bool(params.subtree_mutation_prob) {
            tree = mutate_subtree_with_params(&tree, rng, params);
        } else {
            tree = replace_palette_node(&tree, rng);
        }
        if params.expression_mutation_prob > 0.0 {
            expression_mutate_palette(&mut tree, params.expression_mutation_prob, rng);
        }
        if params.dropout_prob > 0.0 && rng.gen_bool(params.dropout_prob) {
            dropout_node(&mut tree, rng);
        }
        if params.duplication_prob > 0.0 && rng.gen_bool(params.duplication_prob) {
            duplicate_subtree(&mut tree, rng);
        }
        candidate = Genome::new(tree);
        if candidate.palette_range() >= config::PALETTE_MIN_RANGE {
            return candidate;
        }
    }
    candidate
}

fn replace_palette_node(node: &Node, rng: &mut impl Rng) -> Node {
    let def = op_def(node.op);

    match def.arity {
        Arity::Nullary => {
            if node.op == OpCode::PaletteT {
                node.clone()
            } else {
                Node::random_palette(rng)
            }
        }
        Arity::Unary => {
            let mut result = node.clone();
            result.children[0] = replace_palette_node(&node.children[0], rng);
            result
        }
        Arity::Binary => {
            match rng.gen_range(0..3) {
                0 => {
                    let mut result = node.clone();
                    result.children[0] = replace_palette_node(&node.children[0], rng);
                    result
                }
                1 => {
                    let mut result = node.clone();
                    result.children[1] = replace_palette_node(&node.children[1], rng);
                    result
                }
                _ => Node::random_palette(rng),
            }
        }
        Arity::Ternary => {
            if node.op == OpCode::Mix {
                match rng.gen_range(0..3) {
                    0 => {
                        let mut result = node.clone();
                        result.children[0] = replace_palette_node(&node.children[0], rng);
                        result
                    }
                    1 => {
                        let mut result = node.clone();
                        result.children[1] = replace_palette_node(&node.children[1], rng);
                        result
                    }
                    _ => Node::random_palette(rng),
                }
            } else {
                match rng.gen_range(0..4) {
                    0 => {
                        let mut result = node.clone();
                        result.children[0] = replace_palette_node(&node.children[0], rng);
                        result
                    }
                    1 => {
                        let mut result = node.clone();
                        result.children[1] = replace_palette_node(&node.children[1], rng);
                        result
                    }
                    2 => {
                        let mut result = node.clone();
                        result.children[2] = replace_palette_node(&node.children[2], rng);
                        result
                    }
                    _ => Node::random_palette(rng),
                }
            }
        }
    }
}

// ============================================================================
// EXPRESSION MUTATION — per-node operator swap (structure-preserving)
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum MutationContext {
    Spatial,
    Palette,
}

fn random_op_same_arity(current: OpCode, context: MutationContext, rng: &mut impl Rng) -> Option<OpCode> {
    let current_arity = op_def(current).arity;
    let candidates: Vec<OpCode> = OP_REGISTRY.iter()
        .filter(|def| {
            if def.opcode == current { return false; }
            if def.arity != current_arity { return false; }
            match context {
                MutationContext::Spatial => def.opcode != OpCode::PaletteT,
                MutationContext::Palette => !matches!(def.opcode, OpCode::X | OpCode::Y | OpCode::MirrorX | OpCode::MirrorY),
            }
        })
        .map(|def| def.opcode)
        .collect();

    if candidates.is_empty() {
        None
    } else {
        Some(candidates[rng.gen_range(0..candidates.len())])
    }
}

fn expression_mutate_node(node: &mut Node, prob: f64, context: MutationContext, rng: &mut impl Rng) {
    let capped = prob.min(0.30);
    if rng.gen_bool(capped) {
        if let Some(new_op) = random_op_same_arity(node.op, context, rng) {
            node.op = new_op;
            if new_op == OpCode::Const {
                node.value = rng.gen::<f32>();
            }
            if new_op == OpCode::FBM {
                node.c_literal = rng.gen_range(1..=6);
            }
        }
    }
    for child in &mut node.children {
        expression_mutate_node(child, prob, context, rng);
    }
}

pub fn expression_mutate(node: &mut Node, prob: f64, rng: &mut impl Rng) {
    expression_mutate_node(node, prob, MutationContext::Spatial, rng);
}

pub fn expression_mutate_palette(node: &mut Node, prob: f64, rng: &mut impl Rng) {
    expression_mutate_node(node, prob, MutationContext::Palette, rng);
}

// ============================================================================
// TREE TRAVERSAL HELPERS (pre-order indexing)
// ============================================================================

fn count_nodes(node: &Node) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

fn get_node_at(node: &Node, index: usize) -> Option<&Node> {
    if index == 0 { return Some(node); }
    let mut remaining = index - 1;
    for child in &node.children {
        let size = count_nodes(child);
        if remaining < size {
            return get_node_at(child, remaining);
        }
        remaining -= size;
    }
    None
}

fn replace_node_at(node: &mut Node, index: usize, replacement: Node) {
    if index == 0 { *node = replacement; return; }
    let mut remaining = index - 1;
    for child in &mut node.children {
        let size = count_nodes(child);
        if remaining < size {
            replace_node_at(child, remaining, replacement);
            return;
        }
        remaining -= size;
    }
}

// ============================================================================
// DROPOUT — replace a random non-root subtree with a constant
// ============================================================================

fn dropout_node(node: &mut Node, rng: &mut impl Rng) {
    if node.children.is_empty() { return; }

    let idx = rng.gen_range(0..node.children.len());
    let child = &mut node.children[idx];

    if !child.children.is_empty() && !rng.gen_bool(0.5) {
        dropout_node(child, rng);
    } else {
        *child = Node::constant(rng.gen::<f32>());
    }
}

// ============================================================================
// DUPLICATION — copy a subtree to another position (self-crossover)
// ============================================================================

fn duplicate_subtree(node: &mut Node, rng: &mut impl Rng) {
    let total = count_nodes(node);
    if total < 3 { return; }

    let src_idx = rng.gen_range(0..total);

    let mut dst_idx = rng.gen_range(1..total);
    if dst_idx == src_idx {
        dst_idx = if dst_idx + 1 < total { dst_idx + 1 } else { 1 };
    }

    let source_clone = get_node_at(node, src_idx).unwrap().clone();

    let dst_size = count_nodes(get_node_at(node, dst_idx).unwrap());
    let src_size = count_nodes(&source_clone);
    let new_total = total - dst_size + src_size;
    if new_total > config::MAX_TREE_SIZE { return; }

    replace_node_at(node, dst_idx, source_clone);
}

pub fn crossover(a: &Genome, b: &Genome, rng: &mut impl Rng) -> Genome {
    let tree = if rng.gen_bool(0.5) { a.tree() } else { b.tree() };
    Genome::new(tree)
}

pub fn selection(population: &[Genome], rng: &mut impl Rng) -> Genome {
    let size = population.len();
    if size == 1 {
        return population[0].clone();
    }
    population[rng.gen_range(0..size)].clone()
}

impl Genome {
    pub fn tree(&self) -> Node {
        instructions_to_tree(&self.instructions)
    }
}

fn instructions_to_tree(instructions: &[Instruction]) -> Node {
    let mut stack: Vec<Option<Node>> = Vec::new();
    let real_end = instructions.iter().rposition(|i| i.op != OpCode::Const).unwrap_or(0);

    for instr in &instructions[..=real_end] {
        let def = op_def(instr.op);
        let count = def.arity.child_count();
        let indices = [instr.a as usize, instr.b as usize, instr.c as usize];

        // Collect children from stack
        let mut children = Vec::with_capacity(count);
        let mut valid = true;
        for &idx in &indices[..count] {
            if idx < stack.len() {
                if let Some(child) = stack[idx].clone() {
                    children.push(child);
                } else {
                    valid = false;
                    break;
                }
            } else {
                valid = false;
                break;
            }
        }

        if valid {
            stack.push(Some(Node {
                op: instr.op,
                children,
                value: instr.value,
                c_literal: instr.c,
            }));
        }
    }

    stack.last().and_then(|n| n.clone()).unwrap_or(Node::constant(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_arity(node: &Node) -> bool {
        let expected = op_def(node.op).arity.child_count();
        if node.children.len() != expected { return false; }
        node.children.iter().all(verify_arity)
    }

    fn collect_ops(node: &Node, out: &mut Vec<OpCode>) {
        out.push(node.op);
        for child in &node.children {
            collect_ops(child, out);
        }
    }

    fn tree_shape(node: &Node) -> Vec<usize> {
        let mut shape = vec![node.children.len()];
        for child in &node.children {
            shape.extend(tree_shape(child));
        }
        shape
    }

    fn make_test_tree() -> Node {
        // Add(Sin(X), Mul(Y, Const(0.5)))
        Node::binary(
            OpCode::Add,
            Node::unary(OpCode::Sin, Node::terminal(OpCode::X)),
            Node::binary(
                OpCode::Mul,
                Node::terminal(OpCode::Y),
                Node::constant(0.5),
            ),
        )
    }

    // ── Expression mutation tests ────────────────────────────────────────────

    #[test]
    fn expression_mutate_preserves_structure() {
        let mut rng = rand::thread_rng();
        let original = make_test_tree();
        let original_shape = tree_shape(&original);

        for _ in 0..100 {
            let mut tree = original.clone();
            expression_mutate(&mut tree, 1.0, &mut rng);
            assert_eq!(tree_shape(&tree), original_shape);
            assert!(verify_arity(&tree));
        }
    }

    #[test]
    fn expression_mutate_respects_context() {
        let mut rng = rand::thread_rng();
        let spatial_excluded = [OpCode::PaletteT];
        let palette_excluded = [OpCode::X, OpCode::Y, OpCode::MirrorX, OpCode::MirrorY];

        for _ in 0..100 {
            let mut spatial_tree = Node::random_with_depth(&mut rng, 8);
            expression_mutate(&mut spatial_tree, 1.0, &mut rng);
            let mut ops = Vec::new();
            collect_ops(&spatial_tree, &mut ops);
            for &op in &ops {
                assert!(!spatial_excluded.contains(&op), "Spatial context produced PaletteT");
            }
        }

        for _ in 0..100 {
            let mut palette_tree = Node::random_palette_with_depth(&mut rng, 8);
            expression_mutate_palette(&mut palette_tree, 1.0, &mut rng);
            let mut ops = Vec::new();
            collect_ops(&palette_tree, &mut ops);
            for &op in &ops {
                assert!(!palette_excluded.contains(&op), "Palette context produced {:?}", op);
            }
        }
    }

    #[test]
    fn expression_mutate_handles_const_and_fbm() {
        let mut rng = rand::thread_rng();
        // Repeatedly mutate a tree with prob=1.0 and check Const/FBM invariants
        for _ in 0..200 {
            let mut tree = Node::random_with_depth(&mut rng, 6);
            expression_mutate(&mut tree, 1.0, &mut rng);

            fn check_const_fbm(node: &Node) {
                if node.op == OpCode::Const {
                    assert!((0.0..1.0).contains(&node.value),
                        "Const value {} out of range", node.value);
                }
                if node.op == OpCode::FBM {
                    assert!((1..=6).contains(&node.c_literal),
                        "FBM c_literal {} out of range", node.c_literal);
                }
                for child in &node.children {
                    check_const_fbm(child);
                }
            }
            check_const_fbm(&tree);
        }
    }

    #[test]
    fn expression_mutate_zero_prob_is_noop() {
        let mut rng = rand::thread_rng();
        let original = make_test_tree();
        let original_expr = Genome::new(original.clone()).to_expr_string();

        for _ in 0..50 {
            let mut tree = original.clone();
            expression_mutate(&mut tree, 0.0, &mut rng);
            let expr = Genome::new(tree).to_expr_string();
            assert_eq!(expr, original_expr, "Zero prob should be noop");
        }
    }

    // ── Existing mutation/crossover tests ────────────────────────────────────

    #[test]
    fn mutate_subtree_preserves_root_op() {
        let mut rng = rand::thread_rng();
        let params = EvolutionParams { subtree_stop_prob: 0.0, ..Default::default() };
        for _ in 0..100 {
            let tree = Node::random_with_depth(&mut rng, 8);
            let root_op = tree.op;
            let mutated = mutate_subtree_with_params(&tree, &mut rng, &params);
            assert_eq!(mutated.op, root_op, "Root op changed from {:?} to {:?}", root_op, mutated.op);
        }
    }

    #[test]
    fn mutate_subtree_changes_something() {
        let mut rng = rand::thread_rng();
        let params = EvolutionParams { subtree_stop_prob: 0.0, ..Default::default() };
        // Use a tree with Const nodes so subtree mutation has something to randomize
        let original = Node::binary(
            OpCode::Add,
            Node::unary(OpCode::Sin, Node::constant(0.5)),
            Node::binary(OpCode::Mul, Node::constant(0.3), Node::constant(0.7)),
        );
        let original_expr = Genome::new(original.clone()).to_expr_string();

        let mut changed_count = 0;
        for _ in 0..100 {
            let mutated = mutate_subtree_with_params(&original, &mut rng, &params);
            if Genome::new(mutated).to_expr_string() != original_expr {
                changed_count += 1;
            }
        }
        assert!(changed_count > 0, "Subtree mutation never changed the tree in 100 tries");
    }

    #[test]
    fn replace_node_produces_valid_trees() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let tree = Node::random_with_depth(&mut rng, 8);
            let replaced = replace_node(&tree, &mut rng);
            assert!(verify_arity(&replaced), "replace_node produced invalid arity");
            // Roundtrip through Genome
            let genome = Genome::new(replaced);
            let _reconstructed = genome.tree();
        }
    }

    #[test]
    fn crossover_returns_valid_genome() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let a = Genome::new(Node::random_with_depth(&mut rng, 8));
            let b = Genome::new(Node::random_with_depth(&mut rng, 8));
            let child = crossover(&a, &b, &mut rng);
            let tree = child.tree();
            assert!(verify_arity(&tree), "Crossover produced invalid tree");
            let expr = child.to_expr_string();
            let a_expr = a.to_expr_string();
            let b_expr = b.to_expr_string();
            assert!(expr == a_expr || expr == b_expr,
                "Crossover result doesn't match either parent");
        }
    }

    #[test]
    fn selection_returns_population_member() {
        let mut rng = rand::thread_rng();
        let pop: Vec<Genome> = (0..10)
            .map(|_| Genome::new(Node::random_with_depth(&mut rng, 8)))
            .collect();
        let exprs: Vec<String> = pop.iter().map(|g| g.to_expr_string()).collect();

        for _ in 0..100 {
            let selected = selection(&pop, &mut rng);
            let sel_expr = selected.to_expr_string();
            assert!(exprs.contains(&sel_expr), "Selection returned non-member");
        }
    }

    #[test]
    fn mutate_palette_never_produces_spatial_terminals() {
        let mut rng = rand::thread_rng();
        let params = EvolutionParams::default();
        let spatial_ops = [OpCode::X, OpCode::Y, OpCode::MirrorX, OpCode::MirrorY];

        for _ in 0..100 {
            let genome = Genome::new(Node::random_palette_with_depth(&mut rng, 8));
            let mutated = mutate_palette_with_params(&genome, &mut rng, &params);
            let tree = mutated.tree();
            let mut ops = Vec::new();
            collect_ops(&tree, &mut ops);
            for &op in &ops {
                assert!(!spatial_ops.contains(&op),
                    "Palette mutation produced spatial terminal {:?}", op);
            }
        }
    }

    #[test]
    fn mutate_with_params_produces_valid_trees() {
        let mut rng = rand::thread_rng();
        let params = EvolutionParams::default();

        for _ in 0..100 {
            let genome = Genome::new(Node::random_with_depth(&mut rng, 8));
            let mutated = mutate_with_params(&genome, &mut rng, &params);
            let tree = mutated.tree();
            assert!(verify_arity(&tree), "mutate_with_params produced invalid arity");
        }
    }

    // ── Dropout tests ───────────────────────────────────────────────────────

    #[test]
    fn dropout_protects_root() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let mut tree = Node::random_with_depth(&mut rng, 8);
            let root_op = tree.op;
            let root_children = tree.children.len();
            dropout_node(&mut tree, &mut rng);
            assert_eq!(tree.op, root_op, "Dropout changed root op");
            assert_eq!(tree.children.len(), root_children, "Dropout changed root arity");
        }
    }

    #[test]
    fn dropout_never_increases_tree_size() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let mut tree = Node::random_with_depth(&mut rng, 8);
            let before = count_nodes(&tree);
            dropout_node(&mut tree, &mut rng);
            let after = count_nodes(&tree);
            assert!(after <= before, "Dropout grew tree from {} to {}", before, after);
        }
    }

    #[test]
    fn dropout_produces_valid_trees() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let mut tree = Node::random_with_depth(&mut rng, 8);
            dropout_node(&mut tree, &mut rng);
            assert!(verify_arity(&tree), "Dropout produced invalid arity");
        }
    }

    // ── Duplication tests ───────────────────────────────────────────────────

    #[test]
    fn duplication_protects_root() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let mut tree = Node::random_with_depth(&mut rng, 8);
            let root_op = tree.op;
            let root_children = tree.children.len();
            duplicate_subtree(&mut tree, &mut rng);
            assert_eq!(tree.op, root_op, "Duplication changed root op");
            assert_eq!(tree.children.len(), root_children, "Duplication changed root arity");
        }
    }

    #[test]
    fn duplication_produces_valid_trees() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let mut tree = Node::random_with_depth(&mut rng, 8);
            duplicate_subtree(&mut tree, &mut rng);
            assert!(verify_arity(&tree), "Duplication produced invalid arity");
        }
    }

    #[test]
    fn duplication_respects_max_size() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let mut tree = Node::random_with_depth(&mut rng, 10);
            duplicate_subtree(&mut tree, &mut rng);
            let size = count_nodes(&tree);
            assert!(size <= config::MAX_TREE_SIZE,
                "Duplication exceeded MAX_TREE_SIZE: {}", size);
        }
    }

    #[test]
    fn duplication_creates_repeated_subtree() {
        let mut rng = rand::thread_rng();
        let mut found_repeat = false;
        for _ in 0..100 {
            let mut tree = make_test_tree();
            duplicate_subtree(&mut tree, &mut rng);
            let genome = Genome::new(tree);
            let expr = genome.to_expr_string();
            // Check if any non-trivial substring appears more than once
            let parts: Vec<&str> = expr.split(|c: char| c == '(' || c == ')' || c == ',' || c == ' ')
                .filter(|s| s.len() > 1)
                .collect();
            for i in 0..parts.len() {
                for j in (i + 1)..parts.len() {
                    if parts[i] == parts[j] {
                        found_repeat = true;
                        break;
                    }
                }
                if found_repeat { break; }
            }
            if found_repeat { break; }
        }
        assert!(found_repeat, "Duplication never created a repeated pattern in 100 tries");
    }
}
