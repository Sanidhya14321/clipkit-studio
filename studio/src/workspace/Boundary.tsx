import { Component, type ReactNode } from "react";

export class Boundary extends Component<
  { children: ReactNode; name: string },
  { failed: boolean; attempt: number; copied: string }
> {
  state = { failed: false, attempt: 0, copied: "" };
  static getDerivedStateFromError() {
    return { failed: true };
  }
  render() {
    if (this.state.failed)
      return (
        <div className="panel-error" role="alert">
          <strong>{this.props.name} could not render</strong>
          <p>
            Project changes are stored by ck. Reload this view to try again.
          </p>
          <button
            onClick={() =>
              this.setState((s) => ({ failed: false, attempt: s.attempt + 1 }))
            }
          >
            Reload {this.props.name === "Studio" ? "studio" : "panel"}
          </button>
          <button
            onClick={() => {
              const diagnostic = JSON.stringify({
                code: "E_STUDIO_PANEL",
                panel: this.props.name,
                hint: "Reload the view; ck retains committed project revisions.",
              });
              if (!navigator.clipboard) {
                this.setState({ copied: diagnostic });
                return;
              }
              void navigator.clipboard.writeText(diagnostic).then(
                () => this.setState({ copied: "Diagnostics copied" }),
                () => this.setState({ copied: diagnostic }),
              );
            }}
          >
            Copy diagnostics
          </button>
          {this.state.copied && <p role="status">{this.state.copied}</p>}
        </div>
      );
    return (
      <div className="boundary-content" key={this.state.attempt}>
        {this.props.children}
      </div>
    );
  }
}
