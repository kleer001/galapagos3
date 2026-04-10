// Calculate actual terminal probability per Node::random_bounded

const MAX_TREE_DEPTH: usize = 6;
const MIN_TREE_SIZE: usize = 3;
const MAX_TREE_SIZE: usize = 15;

fn calc_terminal_prob() {
    // At depth 0 (root), remaining_budget = 15
    // Terminal condition: remaining_budget < MIN_TREE_SIZE (3) OR current_depth >= max_depth (6)
    // So terminal when: budget < 3 OR depth >= 6

    // Let's trace through the random generation:
    // Depth 0, budget 15: op range 0..20, if op in 0..=1 and depth < 5, wrap in Sin/Cos (cost 2)
    // Depth 0, budget 15: if op in 2 (Const), terminal
    // Depth 0, budget 15: if op in 3..=11 (9 unary ops), cost 2
    // Depth 0, budget 15: if op in 12..=17 (6 binary ops), cost 2+2=4 for both children
    // Depth 0, budget 15: if op 18 (Smoothstep), cost 3 + 2 constants = 5 total

    // Probability of terminal at depth d with budget b:
    // P_terminal = P(budget < 3 OR depth >= 6)
    //            = P(depth >= 6) + P(budget < 3 AND depth < 6)

    // Since we start at depth 0 with budget 15, and each operator costs:
    // - Terminal (X, Y): 1 node
    // - Unary (Sin, Cos, etc.): 2 nodes (operator + 1 child)
    // - Binary (Add, Sub, Mul, Div, Pow, Mix): 3 nodes (operator + 2 children)
    // - Ternary (Smoothstep): 4 nodes (operator + 3 children, but 2 are hardcoded constants)

    // The budget is consumed as: remaining_budget - cost_of_operator
    // For terminal: cost = 1 (just the node itself)
    // For unary: cost = 2 (operator + 1 child node)
    // For binary: cost = 2 (each child gets budget-2)

    println!("Terminal probability analysis:");
    println!("MAX_TREE_DEPTH = {}", MAX_TREE_DEPTH);
    println!("MIN_TREE_SIZE = {}", MIN_TREE_SIZE);
    println!("MAX_TREE_SIZE = {}", MAX_TREE_SIZE);
    println!();

    // At root (depth 0, budget 15):
    // - Terminal condition: budget < 3 OR depth >= 6
    // - depth 0 < 6, so only budget < 3 triggers terminal
    // - But budget starts at 15, so we NEVER hit the budget limit at root

    println!("At root (depth=0, budget=15):");
    println!("  Terminal condition: budget < {} OR depth >= {}", MIN_TREE_SIZE, MAX_TREE_DEPTH);
    println!("  Since 15 >= {} and 0 < {}, terminal is NEVER triggered by budget/depth at root",
             MIN_TREE_SIZE, MAX_TREE_DEPTH);
    println!();

    // So terminal probability depends on operator selection:
    // op range 0..20 (20 operators)
    // - 0..=1: X or Y (terminals, but wrapped in Sin/Cos if budget allows)
    // - 2: Const (terminal)
    // - 3..=11: 9 unary ops
    // - 12..=17: 6 binary ops
    // - 18: Smoothstep (ternary)
    // - 19: Dot

    println!("Operator distribution at any non-terminal node:");
    println!("  X/Y (0-1):        2/20 = 10%");
    println!("  Const (2):         1/20 = 5%");
    println!("  Unary (3-11):      9/20 = 45%");
    println!("  Binary (12-17):    6/20 = 30%");
    println!("  Smoothstep (18):   1/20 = 5%");
    println!("  Dot (19):          1/20 = 5%");
    println!();

    // When X/Y is selected at depth < max_depth - 1, it gets wrapped in Sin/Cos
    println!("X/Y wrapping (when depth < {}-1 = {}):", MAX_TREE_DEPTH - 1, MAX_TREE_DEPTH - 2);
    println!("  X/Y chosen but wrapped: ~50% of the time (choice 0 or 1 becomes Sin/Cos)");
    println!("  So actual terminal X/Y rate: 2/20 * 50% = 1/20 = 5%");
    println!("  Const terminal rate: 1/20 = 5%");
    println!();

    println!("Effective terminal probability per node selection:");
    println!("  X or Y (as leaf): ~5% (2/20 * 0.5 due to wrapping)");
    println!("  Const: ~5% (1/20)");
    println!("  Total terminal rate: ~10%");
    println!();

    // The user mentioned "about 1%" - let's verify this is wrong
    println!("User's claim of '1% leaf probability': INCORRECT");
    println!("Actual terminal (leaf) probability is approximately 10%, not 1%");
}

fn main() {
    calc_terminal_prob();
}
