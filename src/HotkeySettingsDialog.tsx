const dialogLabel = "\u5feb\u6377\u952e\u8bbe\u7f6e";
const inputModeLabel = "\u8f93\u5165\u6a21\u5f0f";
const pushToTalkLabel = "\u6309\u4f4f\u8bf4\u8bdd\u5feb\u6377\u952e";
const toggleDictationLabel = "\u8fde\u7eed\u8f93\u5165\u5feb\u6377\u952e";
const savingLabel = "\u4fdd\u5b58\u4e2d";
const saveLabel = "\u4fdd\u5b58\u5feb\u6377\u952e";
const cancelLabel = "\u53d6\u6d88";
const closeLabel = "\u5173\u95ed\u5feb\u6377\u952e\u8bbe\u7f6e";
const closeIcon = "\u00d7";

interface HotkeySettingsDialogProps {
  pushToTalkHotkey: string;
  toggleDictationHotkey: string;
  isSaving: boolean;
  message: string | null;
  onPushToTalkHotkeyChange: (value: string) => void;
  onToggleDictationHotkeyChange: (value: string) => void;
  onSave: () => void;
  onClose: () => void;
}

export function HotkeySettingsDialog({
  pushToTalkHotkey,
  toggleDictationHotkey,
  isSaving,
  message,
  onPushToTalkHotkeyChange,
  onToggleDictationHotkeyChange,
  onSave,
  onClose,
}: HotkeySettingsDialogProps) {
  return (
    <div className="modal-backdrop" role="presentation">
      <section className="hotkey-dialog" role="dialog" aria-modal="true" aria-label={dialogLabel}>
        <header>
          <span>{inputModeLabel}</span>
          <strong>{dialogLabel}</strong>
          <button className="dialog-close-button" type="button" onClick={onClose} aria-label={closeLabel} title={closeLabel}>{closeIcon}</button>
        </header>
        <label className="field">
          <span>{pushToTalkLabel}</span>
          <input aria-label={pushToTalkLabel} value={pushToTalkHotkey} onChange={(event) => onPushToTalkHotkeyChange(event.target.value)} />
        </label>
        <label className="field">
          <span>{toggleDictationLabel}</span>
          <input aria-label={toggleDictationLabel} value={toggleDictationHotkey} onChange={(event) => onToggleDictationHotkeyChange(event.target.value)} />
        </label>
        {message ? <p className="runtime-message">{message}</p> : null}
        <div className="button-row">
          <button type="button" onClick={onSave} disabled={isSaving}>{isSaving ? savingLabel : saveLabel}</button>
          <button type="button" onClick={onClose}>{cancelLabel}</button>
        </div>
      </section>
    </div>
  );
}
