# PROMPTS.md

## AI Prompts used during the development of this project

### 1.
Can you create me a workflow for the data for this project that fits into these requirements. 
Optional Assignment: See instructions below for Cloudflare AI app assignment. SUBMIT GitHub repo URL for the AI project here. 
(Please do not submit irrelevant repositories.) Optional Assignment Instructions: 
We plan to fast track review of candidates who complete an assignment to build a type of 
AI-powered application on Cloudflare. An AI-powered application should include the following 
components: LLM (recommend using Llama 3.3 on Workers AI), or an external LLM of your choice 
Workflow / coordination (recommend using Workflows, Workers or Durable Objects) User input 
via chat or voice (recommend using Pages or Realtime) Memory or state Find additional 
documentation here. The project is a trip planner where the user submits the place they are 
going and for how long and the ai comes up with a trip plan for them.

### 2. 
Can you explain how cloudflares durable objects work

### 3.
Whats the best way to use cloudflares Workers AI in rust

### 4.
What am I doing wrong in my wrangler config?
⛅️ wrangler 4.45.4 (update available 4.46.0)
─────────────────────────────────────────────
▲ [WARNING] Processing wrangler.toml configuration:


    - "durable_objects.bindings[0]": {"name":"TRIP_SESSION_DO","new_sqlite_classes":"TripSession"}
      - Unexpected fields found in durable_objects.bindings[0] field: "new_sqlite_classes"



X [ERROR] Processing wrangler.toml configuration:


    - "durable_objects.bindings[0]": {"name":"TRIP_SESSION_DO","new_sqlite_classes":"TripSession"}
      - binding should have a string "class_name" field.


🪵  Logs were written to "C:\Users\jfram\AppData\Roaming\xdg.config\.wrangler\logs\wrangler-2025-11-07_04-17-47_011.log"

### 5.
Generate documentation