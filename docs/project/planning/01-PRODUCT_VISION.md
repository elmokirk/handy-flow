---
id: "product-vision"
title: "Product Vision"
type: "planning"
status: "accepted"
version: "1.1"
updated: "2026-08-21"
project: "custom-handy"
baseline_id: "handy-main-2026-08-24-af48dd68"
---


# Product Vision

[[planning/README|← Planning Index]] · [[planning/02-RESEARCH|Research →]]

## Product statement

Create a Windows-first, local-first Handy fork that replaces the core daily Wispr Flow workflow and turns local dictation into a durable personal knowledge source accessible by applications and coding agents.

## Core journeys

| Journey | MVP |
|---|---:|
| Local dictation → paste | Yes |
| Permanent audio + transcript history | Yes |
| Dictionary | Yes |
| Spoken Snippets | Yes |
| Manual Styles | Yes |
| Dictation Transform shortcuts | Yes |
| Local OpenAI-compatible LLM | Yes |
| Searchable History | Yes |
| Markdown/Plain-Text Scratchpad | Yes |
| Markdown/Second-Brain export | Yes |
| Local read-only REST | Yes |
| Local read-only MCP | Yes |
| Rich Text | No |
| Multi-device sync | No |

## Product principles

- **Local-first:** network is optional for core capture.
- **Preserve source:** original audio and engine output are never overwritten.
- **Traceable derivation:** deterministic/AI outputs are versions, not replacements.
- **Fail open:** optional processors cannot destroy a successful transcription.
- **Deterministic first:** Dictionary/Snippets do not require AI.
- **One core, many adapters:** UI/REST/MCP share domain services.
- **Windows-first:** quality optimized for the owner's actual platform.
- **Upstream-friendly where sensible:** generic improvements remain extractable.

## MVP release definition

MVP is not "code complete."

It requires:
- G0–G6 green;
- REST and MCP complete;
- no S0/S1 defect;
- production upgrade/recovery verified;
- 7 consecutive days of owner daily-driver use without needing Wispr for the defined MVP journeys.

See [[planning/05-DELIVERY_PLAN|Delivery Plan]].
