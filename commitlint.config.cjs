module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    // Disable subject-case entirely. The default config-conventional bans
    // sentence-case / start-case / pascal-case / upper-case, but the
    // detector misclassifies legitimate mixed-case technical subjects
    // (e.g. "SDK contract — README + docs.rs include") as start-case
    // because they begin with an acronym. Acronyms (SDK, MSRV, JSON,
    // YAML, HTTP, OIDC, CI, RC, ...) show up regularly in this repo's
    // technical commits; rewording around them is worse than dropping
    // the case rule. Conventional-commits itself doesn't mandate a
    // subject case — that's a config-conventional opinion on top.
    'subject-case': [0],
  },
};
