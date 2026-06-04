# V12 Desktop Compatibility And Release Readiness Design

## Context

V11 is planned to add Windows `SendInput(KEYEVENTF_UNICODE)` insertion with clipboard fallback. After that change, VoxType will have multiple real desktop insertion paths and enough product surface to start hardening beyond feature implementation. The next risk is not another ASR feature; it is confidence across common Windows targets, packaging quality, and repeatable release verification.

Existing docs already call out packaging/release as a medium-risk area: release builds, signing, versioning, release notes, and automatic updates are not designed yet. V12 focuses on compatibility and release readiness before considering TSF.

## Goal

Build a repeatable desktop compatibility and release-readiness layer for VoxType. V12 should produce a maintained compatibility matrix, a manual verification checklist that covers key Windows app targets, release build scripts/checks, and clear go/no-go criteria for when `Auto` insertion can become default.

## Non-Goals

- Do not implement TSF.
- Do not add new ASR providers.
- Do not change the V11 insertion architecture except for compatibility evidence and safe defaults.
- Do not implement auto-update unless release signing/versioning are first documented and verified.
- Do not introduce telemetry or cloud sync.

## Product Behavior

V12 adds no large new end-user workflow by default. Instead, it makes the product safer to distribute and easier to validate. User-visible changes should be limited to polished diagnostics and release metadata where useful: app version, build channel, insertion compatibility status, and clear failure guidance.

The main artifact is a compatibility matrix covering at least:

- VS Code.
- Notepad.
- Browser text fields.
- WeChat or another IME-heavy chat target if available.
- One elevated or permission-boundary target if available.
- Clipboard strategy, SendInput strategy, and Auto fallback behavior.

## Release Readiness

V12 should define and validate:

- Release build command and expected artifacts.
- Versioning policy for desktop builds.
- Release notes template.
- Privacy/sensitive-data scan before release.
- Whether code signing is required before public distribution.
- Whether auto-update is deferred or prepared behind a disabled flag.

## Architecture

The compatibility matrix should live in repository docs and be referenced by harness state. The app may expose app version/build channel through existing diagnostics if that can be done without new UI clutter. Release scripts should wrap existing npm/cargo/Tauri commands rather than inventing a new build system.

## Verification

V12 cannot be marked passing from unit tests alone. It requires automated checks plus manual desktop evidence across the compatibility matrix. Automated checks should verify build metadata, release command smoke checks, and docs JSON validity.

## TSF Decision Gate

V12 should end with a written decision: either SendInput plus clipboard fallback is good enough for the next public release, or TSF deserves a V13 feasibility spike. TSF implementation should not start without this evidence.
