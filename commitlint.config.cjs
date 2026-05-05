module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    // Allow uppercase tokens in subjects so common acronyms (SDK, MSRV,
    // JSON, YAML, HTTP, OIDC, CI, RC, etc.) don't need awkward rewording.
    // The other case restrictions from config-conventional stay
    // (sentence-case, start-case, pascal-case still flagged).
    'subject-case': [2, 'never', ['sentence-case', 'start-case', 'pascal-case']],
  },
};
