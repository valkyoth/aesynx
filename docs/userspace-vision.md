# Aesynx Userspace Vision

Status: design vision

This document captures the userspace direction for Aesynx. The important decision is that Aesynx should not begin by copying Unix userspace. It should be a new OS idea: capability-native, object-native, structured-data-native, WebAssembly-extensible, and AI-assisted.

Unix compatibility is not the default path. A future compatibility service can exist, but it must not define the native userspace model.

## 1. Core Position

Aesynx userspace should not center on:

```text
Unix shell -> fork native process -> text pipe -> parse text
```

It should center on:

```text
aesh shell
  -> launches native components or WASM components
  -> grants explicit capabilities
  -> connects typed streams and object channels
  -> renders rich structured output
  -> records telemetry, provenance, and authority use
```

Text remains useful, but it is not the primary OS data model. Text is a display format, import/export format, and fallback format.

## 2. Userspace Goals

Native Aesynx userspace should be:

- Capability-native: every command receives explicit authority.
- Object-native: commands work with object IDs, object caps, and immutable records.
- Structured-data-native: pipelines pass typed values, streams, and tables.
- WASM-extensible: third-party commands and scripts can run in a sandbox.
- Rust-first: core shell, runtime, and system tools are Rust.
- AI-assisted: autocomplete, explanation, summarization, and query building are integrated but constrained.
- Auditable: commands can explain what they accessed and what authority they used.
- Fast: avoid fork/text-parse overhead as the normal path.
- Portable: WASM components can run across Aesynx architectures and, where useful, on other hosts.

## 3. Execution Model

Do not make WASM replace all native binaries. Use two classes of components.

Native Rust components:

- Core shell.
- Core system tools.
- Device/admin tools.
- Performance-critical tools.
- Trusted services.
- Native runtime services.

WASM components:

- Downloaded tools.
- Plugins.
- Automation.
- Third-party commands.
- Sandboxed extensions.
- Portable apps and scripts.

The shell should be an orchestrator, not a place where arbitrary code receives ambient authority. WASM modules may be lightweight, precompiled, cached, and fast, but they still run in a capability-limited execution context controlled by Aesynx.

## 4. Component Types

```text
Aesynx userspace
|-- aesh
|   |-- parser
|   |-- planner
|   |-- capability prompt/gate
|   |-- structured pipeline engine
|   |-- TUI renderer
|   `-- AI assistant integration
|-- native components
|   |-- built-ins
|   |-- system commands
|   `-- services
|-- WASM components
|   |-- plugins
|   |-- tools
|   `-- automation modules
|-- component store
|   |-- signed objects
|   |-- manifests
|   |-- cached AOT artifacts
|   `-- provenance
`-- value/schema ABI
    |-- typed values
    |-- typed streams
    |-- tables
    `-- errors
```

## 5. Native Shell

The native shell is `aesh`.

Responsibilities:

- Parse commands and pipelines.
- Resolve command objects.
- Check command manifests.
- Ask for or verify required capabilities.
- Connect typed pipeline channels.
- Launch native and WASM components.
- Render structured outputs.
- Provide interactive views.
- Emit telemetry.
- Record provenance.
- Integrate bounded AI assistance.

`aesh` should support familiar command shapes, but not by copying Bash semantics.

Example:

```text
aesh> drivers | where state == "Running" | view
aesh> objects /bin | sort name | view
aesh> fetch-logs --service api | where severity == Error | view
aesh> caps --explain
```

## 6. Structured Pipelines

Traditional shells pass bytes or text. Aesynx pipelines pass typed values.

Traditional:

```text
ls -l | grep "Jun"
```

Aesynx-native:

```text
objects /bin | where kind == "Executable" && modified.month == "June" | view
```

Pipeline values may be:

- Single values.
- Records.
- Tables.
- Streams.
- Object references.
- Capability references.
- Binary blobs.
- Errors.

The shell should know what each command emits and what the next command accepts. If types do not match, the shell should fail early with a useful explanation.

## 7. Aesynx Value Model

The kernel and shell should not literally share Rust's compile-time type system. Rust types are implementation details. Aesynx needs a stable value/schema ABI that Rust, WASM, and future languages can all speak.

Core value model:

```text
Unit
Bool
Int
UInt
Float
String
Bytes
Time
Duration
ObjectId
CapId
List<T>
Record { fields }
Table<T>
Stream<T>
Result<T, Error>
Option<T>
```

Schema metadata:

- Type name.
- Version.
- Field names.
- Field types.
- Optional/required flags.
- Units.
- Display hints.
- Security sensitivity.
- Redaction rules.

This value model should be stable enough for native commands, WASM components, object-store records, telemetry, and AI tools.

## 8. Component ABI

The Aesynx Component ABI is the stable interface for native components and WASM components.

It should define:

- Startup info.
- Arguments.
- Environment.
- Input/output channels.
- Capability handles.
- Object references.
- Error reporting.
- Structured value encoding.
- Telemetry hooks.
- Exit status.

For native Rust components, `aesynx-rt` wraps this ABI.

For WASM components, the runtime maps it to the WASM component interface. WASI concepts can be used where they fit, but Aesynx should not blindly inherit POSIX-like WASI assumptions.

## 9. Capability Manifests

Every component declares the authority it wants.

Example:

```toml
name = "fetch-logs"
kind = "wasm-component"
version = "0.1.0"

[requires]
network = ["service:api-logs"]
read_objects = []
write_objects = []

[outputs]
stream = "LogEntry"
```

The shell and component loader enforce:

- Required capabilities are explicit.
- User or policy grants are explicit.
- Components cannot gain authority by asking another component unless a grant is allowed.
- Authority use is logged.
- Denied authority produces a structured error.

Example run:

```text
aesh> fetch-logs --service api | where severity == Error | view
```

The shell knows:

- What `fetch-logs` may access.
- What type it emits.
- What type `where` expects.
- What type `view` expects.
- What capabilities were used.
- What provenance to record.

## 10. WASM As The Extension And Automation Format

WASM should be the default untrusted extension format.

Why:

- Sandboxed execution.
- Portable across CPU architectures.
- Multiple source languages.
- Cacheable AOT artifacts.
- Clean host-call boundary.
- Natural fit for capability-controlled plugins.

WASM components can be written in:

- Rust.
- Go.
- TypeScript.
- Python-like languages if runtime support exists.
- Other component-model-compatible languages later.

WASM host calls must be capability-checked:

- Read object.
- Write object builder.
- Open service queue.
- Send message.
- Read typed stream.
- Emit typed value.
- Get time.
- Emit telemetry.

No WASM component gets filesystem-style ambient authority by default.

## 11. Native Rust Components

Native Rust is the right path for trusted and performance-sensitive tools.

Examples:

- `aesh`.
- `help`.
- `version`.
- `caps`.
- `objects`.
- `ps`.
- `cores`.
- `drivers`.
- `log`.
- `view`.
- `store`.
- `model`.
- `trace`.

Native components still use capabilities. Native does not mean unrestricted.

## 12. Rich TUI And Views

A modern CLI should not be only monochrome text.

The shell should support:

- Tables.
- Tree views.
- Inspectors.
- Filterable logs.
- Sortable grids.
- Progress dashboards.
- Split panes.
- Inline charts.
- Error sidebars.
- Keyboard and mouse interaction where available.

The `view` command is a first-class renderer:

```text
aesh> log | where severity >= Warn | view
aesh> drivers | view
aesh> trace --boot latest | view
```

If the terminal is limited, output falls back to plain text.

## 13. AI Assistance

AI should be integrated, but bounded.

Allowed AI roles:

- Suggest commands.
- Explain commands.
- Explain capability requests.
- Build filters from natural language.
- Summarize structured output.
- Inspect schemas.
- Highlight anomalies.
- Help write WASM automation modules.
- Explain telemetry and traces.

Forbidden AI roles:

- Gain capabilities by itself.
- Run commands without explicit user approval.
- Bypass object/capability policy.
- Hide authority use.
- Modify system state without a reviewed plan.
- Make irreversible decisions without policy permission.

AI context should be capability-limited. If the assistant cannot read an object, it cannot summarize it.

Example:

```text
aesh> ask "show failed driver restarts from this boot"

plan:
  log --boot current
  where event.kind == DriverRestart && result == Failed
  view

requires:
  read telemetry stream

run? yes/no
```

## 14. Provenance And Audit

Every pipeline should be able to answer:

- Which command objects ran?
- What versions?
- What hashes?
- What capabilities were granted?
- What objects were read?
- What objects were written?
- What network/service endpoints were used?
- Which AI suggestions influenced it?
- What output object was produced?

This makes userspace naturally auditable.

## 15. Object-Native Commands

Commands should work with object IDs and object capabilities naturally.

Examples:

```text
aesh> objects /config
aesh> inspect object:01K...
aesh> store publish ./new-config
aesh> diff object:old object:new | view
aesh> rollback system-root --to object:01J...
```

Text paths can exist as human-friendly names, but internally they resolve through name-index objects to object IDs.

## 16. Error Model

Errors should be structured, not just strings.

```text
Error {
  code: CapabilityDenied,
  message: "fetch-logs cannot access service:api-logs",
  missing_capability: Network("service:api-logs"),
  suggested_action: GrantCapability,
  safe_to_retry: true
}
```

This lets the shell render useful messages and lets AI explain errors without guessing from text.

## 17. First Userspace Milestones

1. Native `aesynx-init` starts.
2. `aesh` starts with text fallback output.
3. Built-ins work:
   - help
   - version
   - echo
   - caps
   - objects
   - ps
   - log
4. Aesynx Value Model exists for simple records and tables.
5. `objects /bin | view` renders a table.
6. Native typed pipeline works.
7. Component manifests declare capabilities.
8. WASM runtime prototype runs a no-authority command.
9. WASM command requests a capability and is denied/granted explicitly.
10. AI command explanation works with no authority escalation.

## 18. Package Management Direction

Native userspace should eventually grow into the package-management model in
[Aesynx Package Manager Roadmap](package-manager-roadmap.md).

The important userspace connection is that package installation, removal,
updates, health repair, and rollback should be ordinary capability-checked
structured operations. `aesh` can expose them as `pkg` commands and typed
pipelines, while future GUI or TUI store clients use the same package daemon
API.

Package lookup can also support lazy command execution: when a command is not
present locally, `aesh` may ask the package service for signed command exports
and present track, publisher, capability, price, and persistence choices before
running or installing anything. This must be policy-controlled and disabled by
default in high-security contexts.

## 19. 1.0 Userspace Target

Minimum 1.0:

- Native `aesynx-init`.
- Native `aesh`.
- Native core commands.
- Text fallback rendering.
- Initial structured values.
- Basic typed tables.
- Capability manifests for commands.
- Object-name lookup.
- Shell telemetry.
- AI-ready command/explanation hooks.

Preferred 1.0:

- `view` TUI for tables/logs.
- WASM no-authority component.
- WASM capability prompt demo.
- Structured pipeline type checking.
- Pipeline provenance log.
- AI-assisted command explanation.

Not required for 1.0:

- Bash.
- POSIX shell semantics.
- Linux binary compatibility.
- Full WASI compatibility.
- Dynamic native shared libraries.
- Browser-grade GUI.
- Cloud AI integration.

## 20. Design Rule

The core userspace rule is:

```text
Aesynx is not Unix-compatible by default.
Aesynx is capability-native, object-native, structured-data-native, WASM-extensible, and AI-assisted.
```
