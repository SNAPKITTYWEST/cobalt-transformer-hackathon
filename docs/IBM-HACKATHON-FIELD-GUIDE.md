2026 HACKATHON FIELD GUIDE · RESEARCHED 26 SEP 2026
Build your COBOL transformer on IBM.
A practical map of the IBM APIs, cloud services, and Z tooling you can actually combine—plus the products to avoid, the free-tier constraints, and a focused five-service build path.

CORE ENGINE: watsonx.ai · COBOL REST API: None public · LITE ALLOWANCE: 300k tokens/mo · BEST DEPLOY: Code Engine

## The five-service stack
01 watsonx.ai inference — Explain COBOL, translate it to Java or Python, and generate tests and business-rule documentation.
02 Embeddings + rerank — Search a code estate semantically, scope changes, and detect duplicate logic.
03 Code Engine — Host the UI or API and run batch transformation jobs with a free tier.
04 Object Storage — Keep source sets, transformed artifacts, and before/after diffs in a portable S3-compatible store.
05 watsonx Orchestrate — Turn analysis, planning, transformation, and validation into an agentic workflow.

The key decision: IBM does not expose a public, hackathon-ready COBOL transformation REST API. Build that capability with watsonx.ai inference. Treat IBM Bob / the former watsonx Code Assistant for Z as a reference architecture unless organizers provide enterprise access.

## A demo flow judges can follow
One visible chain from upload to validated output. Zowe and DBB are optional extensions when real z/OS access exists.
- STEP 01 Ingest — COBOL + copybooks into Object Storage
- STEP 02 Understand — Embeddings find related programs and rules
- STEP 03 Transform — Granite explains and rewrites selected units
- STEP 04 Validate — Generate tests, compare outputs, record diffs
- STEP 05 Deliver — Code Engine UI; Orchestrate runs the flow

## IBM API inventory (25 services)

### watsonx & generative AI
- **watsonx.ai Runtime inference** (TOP PICK, LIVE) — Foundation-model text generation, chat, and streaming with Granite and supported third-party models. Access: REST /ml/v1/text/{generation,chat,chat_stream}; Python ibm-watsonx-ai; Node.js SDK. IAM bearer token + project ID. Use: explain paragraphs, translate COBOL to Java or Python, generate tests, and document business rules. Lite: 300,000 foundation-model tokens/month, 2 requests/second; tuning excluded.
- **watsonx.ai embeddings, rerank & tokenize** (TOP PICK, LIVE) — /ml/v1/text/embeddings, /ml/v1/text/rerank; Python SDK Embeddings. Use: find every program implementing an interest calculation, detect duplicates, and scope change impact.
- **watsonx.ai model catalog** (LIVE) — GET /ml/v1/foundation_model_specs. Examples observed include Granite 3.3 8B Instruct, Granite 4.0 Small, Granite Embedding 278M Multilingual, and supported Llama/Mistral families.
- **watsonx.ai tuning & Prompt Lab** (PAID PATH) — Tuning Studio; /ml/v4/trainings; LoRA/QLoRA in Python SDK 1.2.5+. Not included in the Lite plan. Use few-shot prompting for a free-tier hackathon.
- **watsonx Orchestrate** (TOP PICK, LIVE) — ibm-watsonx-orchestrate ADK + CLI; SaaS; local Developer Edition; runtime REST API. Use: package "analyze → plan → transform → validate" as an agent and embed it by web chat or REST.
- **watsonx.data & watsonx.governance** (ENTERPRISE) — open lakehouse plus model-lifecycle governance. Useful, but enterprise-leaning for a short hackathon.

### COBOL and IBM Z
- **IBM Bob / Bob Premium Package for Z** (PRIVATE PREVIEW, REFERENCE) — IBM sales / early access; no public hackathon API found. Imitate the architecture: estate metadata plus model reasoning.
- **watsonx Code Assistant for Z** (SUPERSEDED, REFERENCE) — VS Code / Eclipse extensions; no public REST API found.
- **Zowe** (WITH z/OS, OPEN SOURCE) — REST interfaces for z/OS datasets, jobs, and USS files. Only relevant with mainframe access.
- **Dependency Based Build (DBB)** (WITH z/OS, OPEN SOURCE) — Groovy build framework that understands COBOL dependencies; runs on z/OS.
- **IBM Z Open Editor** (LIVE) — VS Code extension for COBOL, PL/I, JCL, and REXX.

### Classic Watson
- **watsonx Assistant** (LIVE) — Lite 10,000 messages/month. Add "ask questions about this COBOL program".
- **Watson Discovery** (LIVE) — Lite 1,000 documents/month. Ground transformations in mainframe documentation.
- **Speech to Text / Text to Speech** (LIVE) — voice annotation / narration for the demo.
- **Natural Language Understanding** (DEPRECATED, removal 31 January 2028) — avoid for a new build.
- **Retired Watson APIs** (DO NOT USE) — Language Translator, Tone Analyzer, NLC, Personality Insights, Visual Recognition, Compare & Comply.

### IBM Cloud platform
- **Code Engine** (TOP PICK, FREE TIER) — serverless containers, jobs, functions; scale-to-zero. Deploy the transformer web service and batch jobs.
- **Cloud Object Storage** (TOP PICK, 25GB LITE) — S3 API with HMAC credentials. Store source sets, outputs, diffs.
- **Event Streams** (LITE) — managed Kafka; US South only; 1 partition, 100KB/s, 5 clients.
- **IBM Cloud Databases** (VERIFY PLAN) — Cloudant Lite 1GB and Db2 Lite 200MB reported; verify.
- **Key Protect & Secrets Manager** (LIVE) — keep watsonx credentials out of code.
- **MQ, App ID & API Connect** (CHECK PRICING).

### Specialized & novelty
- **Qiskit & IBM Quantum** (NOVELTY) — ~10 min/month real hardware; no relevance to a COBOL transformer.
- **The Weather Company Data APIs** (COMMERCIAL) — unrelated.

## Start with one grounded call
```
curl -X POST \
  "https://us-south.ml.cloud.ibm.com/ml/v1/text/chat?version=YYYY-MM-DD" \
  -H "Authorization: Bearer <IAM_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{
    "model_id": "ibm/granite-3-3-8b-instruct",
    "project_id": "<PROJECT_ID>",
    "messages": [{
      "role": "user",
      "content": "Explain the business rules in this COBOL paragraph, then propose a Java equivalent and tests."
    }]
  }'
```
```python
from ibm_watsonx_ai.foundation_models import ModelInference

model = ModelInference(
  model_id="ibm/granite-3-3-8b-instruct",
  project_id="<PROJECT_ID>",
  credentials={
    "url": "https://us-south.ml.cloud.ibm.com",
    "apikey": "<API_KEY>"
  }
)

result = model.chat(messages=[{
  "role": "user",
  "content": "Explain this COBOL paragraph."
}])
```
Regional hosts: us-south · eu-de · jp-tok · au-syd. Query the model catalog before hard-coding a model.

## Get access in five moves
01 Create an IBM Cloud Lite account (no credit card, 40+ free services, no time limit).
02 Create an API key in IBM Cloud IAM; exchange it for IAM bearer tokens for REST calls.
03 Provision watsonx.ai Runtime on the Lite plan and create a watsonx.ai Studio project (supplies project_id).
04 Select the matching regional endpoint: US South, Frankfurt, Tokyo, or Sydney.
05 Ask organizers whether sponsored watsonx credits are available.

## Know the edges
- No public COBOL transformer API — use watsonx.ai plus your own parser, prompts, and validation.
- Tuning is outside Lite — use structured prompts and few-shot examples.
- Database freebies are unclear — verify the catalog.
- MQ pricing is unverified.
- Bob for Z needs special access.
- Quantum is not your differentiator.

Research snapshot: 26 September 2026. Verify the linked IBM plan pages before final submission.
