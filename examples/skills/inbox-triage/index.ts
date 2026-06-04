import { skill, llm, secret } from "@asmitdash/encom";

export default skill({
  async run() {
    const token = await secret("GMAIL_TOKEN");
    const res = await fetch(
      "https://api.gmail.com/gmail/v1/users/me/messages?q=is:unread&maxResults=20",
      { headers: { Authorization: `Bearer ${token}` } },
    );
    const list = await res.json();
    const r = await llm.complete({
      system:
        "Sort these emails into urgent / reply-later / archive. Be terse. Output JSON.",
      user: JSON.stringify(list),
    });
    return r.text;
  },
});
