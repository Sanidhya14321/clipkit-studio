// Bundle entry: one composition, "project", drawn by the studio's own ProjectComposition, so
// the preview and the export render the same thing. Size, fps and length come from the
// job's input props (calculateMetadata).
import { Composition, registerRoot } from "remotion";
import {
  ProjectComposition,
  type CompositionProps,
} from "../../studio/src/preview/ProjectComposition";
import { durationInFrames } from "../../studio/src/timeline";

const Root = () => (
  <Composition
    id="project"
    component={ProjectComposition}
    durationInFrames={1}
    fps={30}
    width={1920}
    height={1080}
    defaultProps={{} as CompositionProps}
    calculateMetadata={({ props }) => ({
      durationInFrames: durationInFrames(props.project),
      fps: props.project.settings.fps,
      width: props.project.settings.width,
      height: props.project.settings.height,
    })}
  />
);

registerRoot(Root);
