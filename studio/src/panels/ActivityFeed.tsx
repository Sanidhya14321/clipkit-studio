// One line per applied patch, newest first (spec 5.2). Token usage per step arrives with the meter.
import type { FeedEntry } from "../state";

export function ActivityFeed({ feed }: { feed: FeedEntry[] }) {
  return (
    <section className="panel feed" aria-label="Activity" data-ck-feed>
      <h2>Activity</h2>
      {feed.length === 0 ? (
        <p className="muted">No changes yet.</p>
      ) : (
        <ol>
          {feed.map((e) => (
            <li key={e.rev} data-ck-feed-rev={e.rev} data-ck-author={e.author}>
              <span className="feed-rev">rev {e.rev}</span>{" "}
              <span className="feed-author">{e.author}</span>{" "}
              {e.label || "applied patch"}
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
