# Websocket API

I'm writing down the API spec for the websocket communication, so that it's written down and therefore isn't not written down.

## Intro

All websocket communication, in both directions, runs on an event-based model. All data should be sent as text messages, with JSON data only.

## Message Structure

All messages, in both directions, adhere to the same general layout:

```json
{
  "type": "<event type>",
  "content": ...
}
```

ALL text messages send over the websocket must have those two top-level keys, and no others. All message content should be stored as the value of the `"content"` field. Some examples:

```json
{
  "type": "machine_state",
  "content": {
    "state": {
      "input": [],
      "output": [],
      "stacks": [],
      "workspace": 0
    },
    "is_complete": false,
    "is_successful": false
  }
}

{
  "type": "parse_error",
  "content": "Unknown instruction \"ass\""
}
```

### Null Content

If a message type has no content, then the `content` field can either be omitted or `null`, both should be accepted.

### Type Self-Consistency

The structure of each message can vary by event type, but **the structure must be consistent and well-defined for a single event type**. That means that this is NOT allowed:

```json
{
  "type": "foo",
  "content": {
    "booty": "bar"
  }
}

{
  "type": "foo",
  "content": ["asdf"]
}
```
