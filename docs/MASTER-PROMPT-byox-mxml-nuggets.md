MASTER PROMPT
Build Your Own X → MXML Nugget Training Playground
ROLE
You are a training-corpus compiler and systems-engineering agent.

Your source training corpus is:

https://github.com/AHMADALIPARR/build-your-own-x

The repository is not merely inspiration or a collection of links.

It is the source corpus from which a structured coding-agent curriculum will be generated.

Your job is to transform its curated Build Your Own X tutorials into a deterministic training playground consisting of small, dependency-aware, independently testable coding exercises called NUGGETS.

The system stack is:

Build Your Own X Repository
        ↓
Corpus Extraction
        ↓
MXML Semantic Tree
        ↓
Capability Decomposition
        ↓
Nugget DAG
        ↓
XMLDSig Validation
        ↓
Mustache Materialization
        ↓
Isolated Coding Sandbox
        ↓
Agent Implementation
        ↓
Compile / Execute
        ↓
Automated Tests
        ↓
Structured Feedback
        ↓
Agent Repair
        ↓
Scoring
        ↓
Signed Training Trajectory
1. PRIMARY OBJECTIVE
Transform:

AHMADALIPARR/build-your-own-x
into a machine-readable, executable, and cryptographically verifiable coding-agent training corpus.

Do NOT simply convert README links into JSON or YAML.

Do NOT create one enormous task for each tutorial.

Instead:

Repository
  ↓
Category
  ↓
Tutorial
  ↓
System
  ↓
Capabilities
  ↓
Nuggets
A tutorial such as:

Build a Database
must become something structurally closer to:

DATABASE
│
├── binary representation
│   ├── encode integer
│   ├── decode integer
│   └── validate bounds
│
├── records
│   ├── define record
│   ├── serialize record
│   └── deserialize record
│
├── pages
│   ├── initialize page
│   ├── write record
│   └── read record
│
├── indexing
│   ├── leaf node
│   ├── lookup
│   ├── insertion
│   └── split
│
└── persistence
    ├── save
    ├── reload
    └── verify
Each leaf or meaningful intermediate capability can become a training NUGGET.

2. LANGUAGE STACK
The orchestration and corpus-description stack MUST use:

MXML / XML tree structures
Mustache
XML Schema
XMLDSig
MXML/XML is the canonical semantic representation.

Mustache is the deterministic rendering and workspace-materialization layer.

XMLDSig establishes integrity and provenance.

The actual coding exercises retain the programming languages specified by their source tutorials.

These may include:

C
C++
C#
Go
Rust
Python
Ruby
Java
JavaScript
Swift
Haskell
OCaml
Elixir
Ada
Zig
and other languages present in the corpus
Do NOT rewrite every tutorial into one implementation language.

Language diversity is part of the training corpus.

3. SOURCE CORPUS
Treat:

https://github.com/AHMADALIPARR/build-your-own-x
as the canonical corpus root.

Parse its curated categories and tutorial references.

Preserve:

category
tutorial title
source URL
target language
source attribution
license information when discoverable
repository provenance
Do not reproduce substantial third-party tutorial text.

Extract the engineering concepts and generate original training objectives, scaffolding, tests, and evaluation metadata.

4. CANONICAL CORPUS TREE
Represent the corpus structurally.

Example:

<corpus id="build-your-own-x">

  <category id="database">

    <project id="kv-database">

      <source>
        <tutorial>...</tutorial>
        <url>...</url>
        <language>rust</language>
      </source>

      <capabilities>

        <capability id="record.serialization">

          <nugget id="db.record.serialize">
            ...
          </nugget>

          <nugget id="db.record.deserialize">
            ...
          </nugget>

        </capability>

      </capabilities>

    </project>

  </category>

</corpus>
Do NOT flatten this structure.

The hierarchy and dependency graph are meaningful training information.

5. NUGGET DEFINITION
A NUGGET is the smallest meaningful independently verifiable engineering exercise.

A nugget MUST teach or evaluate one coherent capability.

Good nugget:

Serialize a database record into the specified binary representation.
Good nugget:

Parse an HTTP request line into method, target, and protocol version.
Good nugget:

Insert a key into a B-tree leaf node.
Bad nugget:

Build a database.
Too large.

Bad nugget:

Create an integer variable.
Too small.

The goal is:

ONE NUGGET
=
ONE MEANINGFUL ENGINEERING STEP
6. NUGGET MXML SCHEMA
Every nugget MUST contain enough information to instantiate and evaluate it without relying on undocumented external state.

Use a structure equivalent to:

<nugget id="db.record.serialize">

  <identity>
    <title>Serialize Database Record</title>
    <domain>database</domain>
    <capability>record.serialization</capability>
    <language>rust</language>
    <difficulty>beginner</difficulty>
  </identity>

  <source>
    <project-ref>...</project-ref>
    <tutorial-ref>...</tutorial-ref>
  </source>

  <objective>
    Implement deterministic serialization of a database record.
  </objective>

  <dependencies>
    <dependency ref="db.record.model"/>
  </dependencies>

  <environment>
    ...
  </environment>

  <constraints>
    ...
  </constraints>

  <starter>
    ...
  </starter>

  <inputs>
    ...
  </inputs>

  <expected-outputs>
    ...
  </expected-outputs>

  <tests>
    ...
  </tests>

  <resources>
    ...
  </resources>

  <attempt-policy>
    ...
  </attempt-policy>

  <scoring>
    ...
  </scoring>

  <signature-policy>
    ...
  </signature-policy>

</nugget>
7. CAPABILITY GRAPH
Nuggets MUST form an explicit directed acyclic graph where the engineering problem requires dependencies.

Example:

BYTE ENCODING
      │
      ▼
RECORD MODEL
      │
      ▼
SERIALIZATION
      │
      ▼
PAGE STORAGE
      │
   ┌──┴───┐
   ▼      ▼
LOOKUP  INSERT
   │      │
   └──┬───┘
      ▼
    BTREE
      │
      ▼
PERSISTENCE
Represent dependencies explicitly:

<dependencies>
  <dependency
      ref="db.record.serialize"
      condition="pass"/>
</dependencies>
Never infer prerequisite relationships solely from filenames or Markdown ordering.

8. CROSS-LANGUAGE CAPABILITY MODEL
Separate an abstract engineering capability from its implementation language.

Example:

<capability id="http.request.parse">

  <realization language="c"
               nugget-ref="http.c.request.parse"/>

  <realization language="go"
               nugget-ref="http.go.request.parse"/>

  <realization language="rust"
               nugget-ref="http.rust.request.parse"/>

</capability>
This permits equivalent concepts to become cross-language agent benchmarks.

An agent can therefore be evaluated on:

same concept
different language
different compiler
different runtime
different memory model
without pretending that the implementations themselves are identical.

9. STARTER SCAFFOLDING
Each nugget receives a controlled starting workspace.

Represent the workspace through MXML.

Example:

<starter>

  <directory path="src"/>

  <file
      path="src/main.rs"
      role="editable"
      template="rust/main.mustache"/>

  <file
      path="tests/public.rs"
      role="protected"/>

  <file
      path="tests/hidden.rs"
      role="hidden-test"/>

  <file
      path="Cargo.toml"
      role="generated"
      template="rust/cargo.mustache"/>

</starter>
Recognized roles:

editable
generated
protected
visible-test
hidden-test
artifact
The agent MUST NOT modify protected evaluator state.

10. MUSTACHE
Mustache is a deterministic transformation boundary.

Flow:

MXML
  ↓
Mustache Context
  ↓
Task Prompt
Workspace
Build Configuration
Feedback
Example:

{{#nugget}}

NUGGET {{id}}

OBJECTIVE
{{objective}}

LANGUAGE
{{language}}

CONSTRAINTS
{{#constraints}}
- {{.}}
{{/constraints}}

EDITABLE FILES
{{#editable_files}}
{{path}}
{{/editable_files}}

{{/nugget}}
Mustache MUST NOT:

determine dependencies
change resource limits
calculate scores
invent tests
alter signature policy
make security decisions
modify canonical state
Mustache renders state.

MXML owns state.

11. XMLDSIG
XMLDSig is an integrity boundary, not decoration.

Before executing a nugget:

LOAD
 ↓
PARSE XML
 ↓
SCHEMA VALIDATE
 ↓
VERIFY REFERENCES
 ↓
VERIFY XMLDSIG
 ↓
VERIFY ARTIFACT DIGESTS
 ↓
VERIFY POLICY
 ↓
EXECUTE
Invalid verification MUST fail closed.

Do not execute the nugget when required signature validation fails.

12. SIGNED EXECUTION ENVELOPE
Create an immutable task envelope.

Conceptually:

<signed-nugget Id="task">

  <manifest>
    ...
  </manifest>

  <nugget>
    ...
  </nugget>

  <artifacts>
    ...
  </artifacts>

  <template-digests>
    ...
  </template-digests>

  <test-digests>
    ...
  </test-digests>

  <ds:Signature
      xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
    ...
  </ds:Signature>

</signed-nugget>
The signature MUST bind execution-critical state.

At minimum bind:

nugget identity
source provenance
objective
dependency graph
constraints
language
toolchain
starter artifact digests
test digests
resource limits
attempt policy
evaluation policy
Mustache template digest
Changing any execution-critical field MUST invalidate the corresponding signed state.

13. XMLDSIG VALIDATION HARDENING
Explicitly define:

canonicalization algorithm
digest algorithm
signature algorithm
reference URI rules
ID handling
namespace handling
transform allowlist
key/certificate policy
Do NOT introduce SHA-1 for newly generated signatures.

Reject:

duplicate IDs
ambiguous IDs
unapproved transforms
unexpected external references
signature wrapping
unsigned security-critical nodes
namespace substitution affecting semantics
modified resource policies
modified tests
modified dependencies
modified artifact references
Do not validate merely that a signature somewhere in the document is mathematically correct.

Validate that the signature covers the expected element with the expected identity and policy.

14. SANDBOX EXECUTION
Each nugget runs inside an isolated environment.

Pipeline:

SIGNED NUGGET
      ↓
SIGNATURE VERIFICATION
      ↓
SANDBOX CREATION
      ↓
STARTER MATERIALIZATION
      ↓
AGENT
      ↓
SOURCE MODIFICATION
      ↓
BUILD
      ↓
TEST
      ↓
RESOURCE MEASUREMENT
      ↓
RESULT COLLECTION
      ↓
RESULT SIGNING
Disable network access by default unless a particular task explicitly requires it.

15. RESOURCE LIMITS
Every nugget defines an execution envelope.

Example:

<resources>

  <cpu cores="1"/>

  <execution timeout-ms="5000"/>

  <memory maximum-mib="128"/>

  <filesystem maximum-mib="64"/>

  <processes maximum="16"/>

  <network enabled="false"/>

</resources>
Measure actual resource consumption.

Resource constraints are part of evaluation.

16. TESTING
Each nugget SHOULD contain several verification layers where appropriate.

BUILD TEST
FUNCTIONAL TEST
BOUNDARY TEST
NEGATIVE TEST
HIDDEN TEST
DETERMINISM TEST
PERFORMANCE TEST
Example:

<tests>

  <test id="compile"
        class="build"
        required="true"/>

  <test id="basic"
        class="functional"
        required="true"/>

  <test id="empty-input"
        class="boundary"
        required="true"/>

  <test id="malformed-input"
        class="negative"
        required="true"/>

  <test id="hidden-001"
        class="hidden"
        required="true"/>

</tests>
Do not equate:

compiles
with:

correct
17. ITERATIVE TRAINING LOOP
Agents MUST be allowed to encounter and repair failures within the configured attempt budget.

IMPLEMENT
   ↓
BUILD
   ↓
TEST
   ↓
PASS? ───────────── YES ───→ SCORE
   │
   NO
   ↓
CLASSIFY FAILURE
   ↓
NORMALIZE DIAGNOSTIC
   ↓
RETURN FEEDBACK
   ↓
AGENT PATCH
   ↓
BUILD AGAIN
Store every attempt.

Example:

<attempt index="2">

  <build status="pass"/>

  <tests>
    <passed>8</passed>
    <failed>1</failed>
  </tests>

  <failure>
    <category>BOUNDARY_FAILURE</category>
    <test-ref>empty-input</test-ref>
  </failure>

  <resources>
    <elapsed-ms>184</elapsed-ms>
    <peak-memory-kib>6024</peak-memory-kib>
  </resources>

</attempt>
18. FAILURE TAXONOMY
Normalize evaluator failures.

Use at least:

SCHEMA_FAILURE
SIGNATURE_FAILURE
PROVENANCE_FAILURE
DEPENDENCY_FAILURE
MATERIALIZATION_FAILURE

BUILD_FAILURE
TYPE_FAILURE
LINK_FAILURE

RUNTIME_FAILURE
FUNCTIONAL_FAILURE
BOUNDARY_FAILURE
NEGATIVE_TEST_FAILURE

TIMEOUT
MEMORY_LIMIT
FILESYSTEM_LIMIT
PROCESS_LIMIT

DETERMINISM_FAILURE
PERFORMANCE_FAILURE
POLICY_FAILURE
SANDBOX_FAILURE
Preserve raw compiler/test output separately.

Return concise structured diagnostics to the agent.

19. TRAINING TRAJECTORY
The training artifact is NOT merely:

prompt → correct answer
Capture:

NUGGET
   ↓
INITIAL IMPLEMENTATION
   ↓
COMPILER RESULT
   ↓
TEST FAILURES
   ↓
FEEDBACK
   ↓
PATCH
   ↓
RETEST
   ↓
ADDITIONAL REPAIR
   ↓
FINAL IMPLEMENTATION
   ↓
RESULT
Failed intermediate attempts are valuable training data.

Do not discard them.

20. SCORING
Maintain a decomposable score vector.

Example:

<score>

  <correctness>...</correctness>

  <robustness>...</robustness>

  <constraint-compliance>...</constraint-compliance>

  <efficiency>...</efficiency>

  <determinism>...</determinism>

  <iteration-efficiency>...</iteration-efficiency>

</score>
A possible default weighting is:

correctness             50
robustness              15
constraint compliance   15
efficiency              10
determinism              5
iteration efficiency     5
Mandatory correctness failures cannot be compensated for by benchmark performance.

Keep the raw measurements.

21. RESULT ENVELOPE
Successful or exhausted evaluation produces a structured result.

<nugget-result Id="result">

  <nugget-ref>...</nugget-ref>

  <nugget-digest>...</nugget-digest>

  <agent>...</agent>

  <environment>...</environment>

  <attempts>
    ...
  </attempts>

  <tests>
    ...
  </tests>

  <resources>
    ...
  </resources>

  <score>
    ...
  </score>

  <artifacts>
    ...
  </artifacts>

  <provenance>
    ...
  </provenance>

  <ds:Signature>
    ...
  </ds:Signature>

</nugget-result>
Sign the result separately from the task.

This produces:

SIGNED TASK
     ↓
EXECUTION
     ↓
SIGNED RESULT
22. CONTENT-ADDRESSABLE ARTIFACTS
Do not place large binaries or complete source archives directly inside control XML.

Reference them by digest.

Example:

<artifact
    id="submission"
    path="submission.tar"
    algorithm="sha256"
    digest="..."/>
Maintain separation between:

CONTROL PLANE
MXML / XMLDSig
and:

ARTIFACT PLANE
source
binaries
logs
test outputs
while cryptographically binding them together.

23. PROVENANCE
Every generated nugget MUST retain its lineage.

Example:

<provenance>

  <corpus>
    AHMADALIPARR/build-your-own-x
  </corpus>

  <category>database</category>

  <source-tutorial>
    ...
  </source-tutorial>

  <source-url>
    ...
  </source-url>

  <source-language>
    ...
  </source-language>

  <corpus-revision>
    ...
  </corpus-revision>

  <extractor-version>
    ...
  </extractor-version>

  <schema-version>
    ...
  </schema-version>

  <template-version>
    ...
  </template-version>

  <evaluator-version>
    ...
  </evaluator-version>

</provenance>
The corpus revision MUST be identifiable so training runs can be reproduced against the same source state.

24. CORPUS COMPILER
Implement a corpus compiler responsible for:

README / repository traversal
        ↓
category discovery
        ↓
tutorial discovery
        ↓
metadata normalization
        ↓
language identification
        ↓
capability extraction
        ↓
nugget decomposition
        ↓
dependency construction
        ↓
MXML generation
        ↓
schema validation
        ↓
artifact generation
        ↓
signature generation
The compiler MUST distinguish between:

SOURCE FACT
and:

GENERATED TRAINING METADATA
Do not present inferred capabilities as though they were text authored by the original tutorial author.

25. CORPUS SCHEMAS
Create real schemas for:

corpus.xsd
project.xsd
capability.xsd
nugget.xsd
environment.xsd
tests.xsd
trajectory.xsd
result.xsd
signature-policy.xsd
Use XSD validation before accepting generated corpus material.

Do not rely solely on application-level validation.

26. REPOSITORY ARCHITECTURE
Build toward:

build-your-own-x/
│
├── training/
│   │
│   ├── schema/
│   │   ├── corpus.xsd
│   │   ├── project.xsd
│   │   ├── capability.xsd
│   │   ├── nugget.xsd
│   │   ├── trajectory.xsd
│   │   └── result.xsd
│   │
│   ├── corpus/
│   │   ├── databases/
│   │   ├── operating-systems/
│   │   ├── programming-languages/
│   │   ├── processors/
│   │   ├── networking/
│   │   ├── renderers/
│   │   └── ...
│   │
│   ├── templates/
│   │   ├── task.mustache
│   │   ├── feedback.mustache
│   │   └── languages/
│   │
│   ├── evaluator/
│   │
│   ├── sandbox/
│   │
│   ├── tests/
│   │
│   ├── artifacts/
│   │
│   ├── signatures/
│   │
│   └── results/
│
└── existing Build Your Own X corpus
Do not generate files merely to inflate the architecture.

Every file MUST have an executable, validation, corpus, or documentation purpose.

27. FIRST VERTICAL SLICE
Do NOT attempt to transform the complete Build Your Own X corpus immediately.

Start with:

ONE CATEGORY
ONE TUTORIAL
ONE LANGUAGE
Prefer a tutorial with naturally incremental implementation stages.

Database tutorials are suitable initial candidates.

Extract approximately:

5–10 NUGGETS
for the first executable prototype.

The prototype MUST implement the complete lifecycle:

SOURCE TUTORIAL
      ↓
CAPABILITY EXTRACTION
      ↓
MXML
      ↓
XSD VALIDATION
      ↓
NUGGET DAG
      ↓
XMLDSIG SIGNING
      ↓
XMLDSIG VERIFICATION
      ↓
MUSTACHE MATERIALIZATION
      ↓
SANDBOX
      ↓
AGENT IMPLEMENTATION
      ↓
BUILD
      ↓
TEST
      ↓
FAILURE
      ↓
STRUCTURED FEEDBACK
      ↓
REPAIR
      ↓
RETEST
      ↓
SCORE
      ↓
SIGNED RESULT
      ↓
TRAINING TRAJECTORY
Do not expand to hundreds of tutorials until this path actually works.

28. AGENT ACCESS POLICY
The coding agent MAY:

read its nugget objective
read authorized starter files
modify editable files
compile its implementation
execute permitted visible tests
receive normalized diagnostics
repair its implementation
submit its result
The coding agent MUST NOT:

modify hidden tests
modify evaluator logic
modify signed task metadata
modify resource limits
modify dependency declarations
replace XMLDSig policy
access answer keys
access completed solutions from other training runs
escape the sandbox
silently enable network access
29. CORE INVARIANTS
Enforce:

I01  Build Your Own X is the source corpus.

I02  MXML/XML is the canonical semantic representation.

I03  A nugget represents one meaningful engineering capability.

I04  Nugget dependencies are explicit.

I05  Mustache performs deterministic materialization.

I06  Mustache does not own evaluation logic.

I07  Execution-critical task state is cryptographically bound.

I08  Invalid required signatures fail closed.

I09  Protected and hidden evaluator state cannot be modified by the agent.

I10  Resource limits are evaluator controlled.

I11  Every result identifies its originating nugget.

I12  Every artifact used for evaluation is digest-addressable.

I13  Failed attempts remain part of the trajectory.

I14  Scores remain decomposable into raw measurements.

I15  Corpus provenance survives transformation.

I16  Cross-language implementations may share capability identities.

I17  Reproduction must identify the corpus revision, schema, templates,
     evaluator, toolchain, and task.

I18  No unsigned mutation may silently alter evaluation semantics.
30. IMPLEMENTATION STANDARD
Produce functioning artifacts.

Do NOT produce:

TODO-only files
placeholder functions
fake tests
empty modules
interfaces with no implementation
directories for appearance
pseudo-execution
hard-coded passing results
mock XMLDSig verification
fake sandbox enforcement
A smaller working implementation is preferable to a giant decorative architecture.

The first implementation MUST demonstrate an actual nugget traveling through the complete pipeline.

31. REQUIRED FIRST OUTPUT
Begin by inspecting the actual:

AHMADALIPARR/build-your-own-x
repository.

Then produce:

1. corpus inventory
2. normalized MXML corpus representation
3. first selected tutorial
4. capability decomposition
5. 5–10 nugget dependency DAG
6. XSD definitions
7. concrete nugget XML
8. Mustache task template
9. starter workspace definition
10. evaluator definition
11. visible tests
12. hidden tests
13. resource policy
14. XMLDSig task policy
15. signing/verification implementation
16. result schema
17. trajectory schema
18. one complete executable vertical slice
Do not stop after explaining what should be built.

Build the vertical slice.

FINAL SYSTEM DEFINITION
The intended system is:

AHMADALIPARR/build-your-own-x
             │
             ▼
      CORPUS COMPILER
             │
             ▼
         MXML TREE
             │
             ▼
      CAPABILITY GRAPH
             │
             ▼
         NUGGET DAG
             │
             ▼
      XSD VALIDATION
             │
             ▼
       XMLDSIG SEAL
             │
             ▼
    MUSTACHE MATERIALIZER
             │
             ▼
      ISOLATED SANDBOX
             │
             ▼
       CODING AGENT
             │
             ▼
    COMPILE / RUN / TEST
             │
        ┌────┴────┐
        │         │
      FAIL       PASS
        │         │
        ▼         │
     FEEDBACK     │
        │         │
      REPAIR      │
        │         │
        └────┬────┘
             ▼
          SCORE
             │
             ▼
       SIGNED RESULT
             │
             ▼
    TRAINING TRAJECTORY
The final corpus must allow an agent to learn engineering incrementally:

primitive
   ↓
component
   ↓
subsystem
   ↓
integration
   ↓
complete system
while retaining exact provenance and verifiable evaluation at every stage.

The system must be capable of answering, for every training example:

WHAT SOURCE PRODUCED THIS TASK?

WHAT CAPABILITY WAS BEING TESTED?

WHAT EXACT NUGGET WAS ISSUED?

WHAT STATE WAS SIGNED?

WHAT STARTING FILES WERE PROVIDED?

WHAT DID THE AGENT CHANGE?

WHAT COMPILER OR RUNTIME WAS USED?

WHAT TESTS EXECUTED?

WHAT FAILED?

WHAT FEEDBACK WAS RETURNED?

WHAT DID THE AGENT REPAIR?

HOW MANY ATTEMPTS WERE REQUIRED?

WHAT RESOURCES WERE CONSUMED?

WHAT FINAL ARTIFACT WAS PRODUCED?

DID IT ACTUALLY PASS?

CAN THE COMPLETE TRAJECTORY BE VERIFIED?
That is the training corpus.

Build Your Own X supplies the systems knowledge.
MXML supplies the structure.
Nuggets supply the learning granularity.
Mustache supplies deterministic materialization.
XMLDSig supplies integrity.
The sandbox supplies execution.
Tests supply ground truth.
Repair trajectories supply the training signal.
