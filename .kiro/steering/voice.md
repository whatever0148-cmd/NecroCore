---
inclusion: fileMatch
fileMatchPattern: ["README.md", "deliverables/**/*.md"]
---

# Writing for judges

The README and everything in `deliverables/` is read by Kiroween judges. Write it
the way a student explains their own project to a friend.

- First person ("I built", "we found"), short sentences, plain words.
- No em dashes and no semicolons. Use a full stop or a comma instead.
- No buzzwords: leverage, seamless, robust, cutting-edge, game-changer,
  revolutionize, unlock, empower, delve, synergy, state-of-the-art,
  next-generation, groundbreaking, supercharge, elevate, world-class.
- Every Halloween label sits next to its technical term: Ghost (unreachable
  code), Vampire (unbounded accumulator), Zombie (non-terminating loop).
- Say plainly what does not work yet and where Kiro or the scanner fell short.

The `judges-voice` hook runs `scripts/hooks/hook.py voice` after every write to
these files and returns any broken line to the agent.
