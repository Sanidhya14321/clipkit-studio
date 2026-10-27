# Video motion assets and instructions

This is the entry point for animation assets used to direct and create videos. It contains actual adopted source, frame-clock adapters, and populated usage cards. It is not a stock-photo collection.

- `catalog.json`: searchable selection metadata; every entry explains when, why and how to adapt it.
- `instructions/`: per-asset guidance, prompt adaptation, timing, editability and rights.
- `sources/`: selected original Clip Library source/dependency closure and retained Magic UI/Animata originals with notices.
- `adapters/web.tsx`: frame-seekable Magic UI, Animata and Paper Shader integrations.
- Existing component APIs/defaults and checksummed source inventory are under `packages/assets/components/`.

There are **12 usable components and 9 reference techniques**. Reference cards are not counted as insertable assets. React Bits entries provide mechanism instructions; its code is not redistributed as a pack. The awesome-react-components repository is a discovery index, not a dependency collection to install wholesale.

## Agent selection pipeline

Brief → story/copy and visual thesis → search a small relevant shortlist → read 1–3 usage cards/APIs → choose mechanisms and explain their purpose → frame choreography → real three-second proof → human approval → production → bounded revision.

Use `library_list` with a query and kind `component` or `reference`. Read `library_inspect` for the usage card and exact API; `source=true` loads actual code only when needed. `motion_read` also accepts a catalog ID. An original composition can use zero catalog components. Do not select one asset per sentence or let the catalog prescribe the film’s layout.

Add your own footage/audio/images in another project subfolder such as `assets/media/`. Curated instructions and source folders are grouped, not shown as hundreds of duplicate source cards. New external executable code still requires review, a deterministic adapter and appropriate rights.
