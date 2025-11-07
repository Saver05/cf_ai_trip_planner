### README.md

### cf_ai_trip_planner

>An AI powered trip planner built on Cloudflare edge network.


## Overview

> This trip planner generates and stores AI generated travel itineraries.
> The users can
> - Generate multi day trip plans with a given duration and destination
> - View the itinerary with daily breakdowns
> - Chat with the AI agent to get more details 
> - Retrieve their saved trip with unique id after leaving the page
> 
> What it can not do:
> - Once generated itinerary cannot be edited or deleted
> - The AI will hallucinate random facts about you, the more context you give it in the questions you ask the more accurate it will be
> - DO NOT put anything sensitive in here, this was a project for fun and learning not for production use and all chats will be saved.

## How to use

You can go to this [link](https://cf-ai-trip-planner.jframs.workers.dev/) to use the app.

or to run it locally you can run the following commands with rust installed:
```
rustup target add wasm32-unknown-unknown
npx wrangler init --no-deploy
npx wrangler config
npx wrangler project set-name cf_ai_trip_planner
npx wrangler project set-main build/index.js
npx wrangler project set-compatibility-date 2025-11-06
npx wrangler kv namespace create USER_PREFERENCES
npx wrangler d1 create TripPlanner
npx wrangler d1 execute TripPlanner --file=./schema.sql 
npx wrangler deploy --new-class TripSession --binding TRIP_SESSION_DO
npx wrangler secret put CF_ACCOUNT_ID
npx wrangler secret put AI_MODEL
npx wrangler secret put CF_API_TOKEN
cargo install -q worker-build && worker-build --release
npx wrangler dev
```
