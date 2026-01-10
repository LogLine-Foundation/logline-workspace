# Paper IV: TDLN — The Logical Atom & Compiler Specification

## 1. 0 Abstract 
**TDLN (Truth-Determining Language Normalizer)** is the **Semantic Instruction Set Architecture (S-ISA)** for the verifiable agent economy. It addresses the "Tower of Ambiguity" problem in modern computing, where human intent degrades as it traverses layers of opaque software abstraction.

TDLN functions as a deterministic compiler pipeline. It accepts high-level intent (via Natural Language or a strict Domain-Specific Language) and transmutes it into a **Canonical Abstract Syntax Tree (Core AST)**. This AST is versioned, hash-addressable, and mathematically isomorphic to the original intent.

This paper formally specifies the **TDLN DSL Grammar**, the **Core AST Schema**, the **Canonicalization Algorithms**, and the **Proof-Carrying Translation** protocol. It establishes the mathematical foundation for "Law is Code, " ensuring that the rules governing autonomous agents are are fixed, calculable physics.

## 2. 0 Architectural Philosophy: The Refraction of Intent 
TDLN operates on the principle of **Semantic Refraction**. Just as a prism splits white light into precise wavelengths, TDLN splits vague semantic intent into precise, executable atoms.

The architecture is defined by a strict separation of concerns: 
1. **The Ingress (The Lens): ** Accepts ambiguous input (NL) or structured input (DSL). 
2. **The Compiler (The Prism): ** Normalizes, sorts, and solidifies the logic into a rigid structure. 
3. **The Runtime (The Screen): ** Executes *only* the compiled, signed artifacts.

**The Security Invariant: ** 
> *Data is never instructions. The runtime engine structurally lacks the capacity to evaluate raw text or ambiguous tokens. *

## 3. 0 The Semantic ISA: Core AST Specification 
The **Core AST** is the "Machine Code" of the TDLN system. It is a platform-agnostic, JSON-serializable tree structure that serves as the single source of truth.

### 3. 1 The Primitive Node Types 
The AST is composed of strictly typed nodes defined by the following TypeScript interfaces (from the TDLN Core Spec v0. 1):

```typescript 
// The fundamental atom of decision 
interface PolicyBit extends TDLNNode { 
 node_type: "policy_bit"; 
 id: string; // UUID v4 
 name: string; // e. g. , "is_premium_user" 
 condition: Expression; // The logic gate 
 fallback: boolean; // Default secure state (Fail-Closed) 
}

// Wiring for complex logic 
interface PolicyComposition extends TDLNNode { 
 node_type: "policy_composition"; 
 composition_type: "SEQUENTIAL" | "PARALLEL" | "CONDITIONAL"; 
 policies: string[]; // References to PolicyBit IDs 
 aggregator? : Aggregator; // Logic for parallel resolution 
}

// The complete, deployable artifact 
interface SemanticUnit extends TDLNNode { 
 node_type: "semantic_unit"; 
 policies: (PolicyBit | PolicyComposition)[]; 
 inputs: Parameter[]; // The context required (the "pins") 
 source_hash: string; // BLAKE3 hash of canonical content 
} 
```

### 3. 2 The Expression Language 
To ensure security and decidability, the expression language is **Turing-Incomplete**. It forbids loops and recursion, preventing Halting Problem exploits.

* **Binary Ops: ** `AND`, `OR`, `EQ`, `NEQ`, `GT`, `LT`, `IN` 
* **Unary Ops: ** `NOT`, `EXISTS` 
* **Context Refs: ** `user. balance`, `transaction. risk_score` 
* **Literals: ** Strings, Integers, Booleans (No Floats)

## 4. 0 The TDLN DSL: Deterministic Grammar 
While the AST is for machines, the **TDLN DSL** is for humans. It provides a readable, formal grammar that compiles 1: 1 into the AST.

### 4. 1 Formal Grammar (EBNF) 
```ebnf 
policy: = '@policy' IDENT description? condition+ composition? 
condition: = 'when' IDENT ': ' expression 
expression: = term (OPERATOR term)* 
term: = literal | context_ref | func_call | '(' expression ')' 
composition: = 'compose' aggregator '(' IDENT (', ' IDENT)* ')' 
aggregator: = 'all' | 'any' | 'majority' | 'weighted' 
```

### 4. 2 Example Source Code 
```tdln 
@policy secure_transfer 
@description "Allow transfer if user is verified AND balance sufficient"

when is_verified: 
 user. kyc_status \== "verified"

when has_funds: 
 user. balance >= transaction. amount

# The Composition (Logical AND) 
compose all(is_verified, has_funds) \-> allow_transfer 
```

## 5. 0 The Compiler Pipeline 
The transformation from text to truth occurs in four rigorous stages.

### 5. 1 Stage 1: Semantic Extraction (Hybrid) 
* **Path A (Deterministic): ** The DSL Parser processes the formal grammar defined above. Result: 100% reproducible. 
* **Path B (Probabilistic): ** An LLM extracts entities and logic from Natural Language into a JSON intermediate. This is then passed to the deterministic builder. *Crucially, the user must sign the extraction, not the raw text. *

### 5. 2 Stage 2: AST Construction 
The extracted logic is instantiated into `PolicyBit` and `PolicyComposition` objects. UUIDs are assigned, and type-checking is performed against the `Parameter` definitions.

### 5. 3 Stage 3: Canonicalization ($\\rho$) 
This is the core innovation. To ensure $Hash(A) \= Hash(B)$, we apply aggressive normalization: 
1. **Lexicographical Sorting: ** All keys, parameter lists, and policy arrays are sorted by name/ID. 
2. **Expression Simplification: ** Boolean algebra reduction (e. g. , `A AND True` $\\to$ `A`). 
3. **Whitespace/Type Normalization: ** Unicode NFC, Integers only.

### 5. 4 Stage 4: Proof Generation 
The compiler emits a **Translation Proof** alongside the AST. This allows third-party auditing of the compilation process.

```typescript 
interface TranslationProof { 
 proof_type: "translation"; 
 source_hash: string; // Hash of input text 
 target_hash: string; // Hash of final Core AST 
 steps: TranslationStep[]; // Log of transformations 
 signature: string; // Compiler Authority Signature 
} 
```

## 6. 0 Formal Verification: The Semantic Isomorphism Theorem 
We assert that the TDLN compiler preserves the logical truth of the input.

**Theorem (Semantic Isomorphism): ** 
Let $S$ be a valid policy in the source grammar $G$, and $T$ be the transformation function to the Core AST. For any evaluation context $C$: 
$$ Eval_{DSL}(S, C) \\equiv Eval_{AST}(T(S), C) $$

**Proof Strategy: ** 
1. **Bijective Operator Mapping: ** Every operator in the DSL ($\\land, \\lor, \\neg$) maps to a unique, semantically identical node in the AST schema. 
2. **Structural Homomorphism: ** The recursive descent parser preserves the precedence and nesting structure of the logical expressions. 
3. **Canonical Invariance: ** The canonicalization function $\\rho$ permutes elements (sorting) but does not alter logical topology (dependencies).

## 7. 0 Performance and Efficiency 
The TDLN compiler and runtime are optimized for high-frequency agent interactions.

**Benchmarks (Rust Implementation): ** 
* **Cold Start: ** 4ms (vs 45ms for OPA). 
* **Evaluation: ** 0. 3 $\\mu$s per Policy Bit. 
* **Memory Footprint: ** \~2KB per Policy Unit.

This efficiency enables "Policy-per-Packet" filtering in network layers (SIRP) without introducing perceptible latency.

## 8. 0 Conclusion: The Standard for Truth 
TDLN represents the maturation of policy from a document to an artifact.

In the LogLine architecture, the **Core AST** is the unforgeable token of intent. It allows an AI agent to prove, cryptographically, that the action it is taking is exactly the action authorized by its human principal. It transforms the "Spirit of the Law" (Intent) and the "Letter of the Law" (Code) into a single, indivisible, and computable reality. 
