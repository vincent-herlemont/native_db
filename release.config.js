module.exports = {
  branches: ['main', 'weekly-release'],
  tagFormat: '${version}',
  plugins: [
    ['@semantic-release/commit-analyzer', {
      releaseRules: [
        {breaking: true, release: 'minor'},
        {revert: true, release: 'patch'},
        {type: 'feat', release: 'minor'},
        {type: 'fix', release: 'patch'},
        {type: 'perf', release: 'patch'},
        {emoji: ':racehorse:', release: 'patch'},
        {emoji: ':bug:', release: 'patch'},
        {emoji: ':penguin:', release: 'patch'},
        {emoji: ':apple:', release: 'patch'},
        {emoji: ':checkered_flag:', release: 'patch'},
        {tag: 'BUGFIX', release: 'patch'},
        {tag: 'FEATURE', release: 'minor'},
        {tag: 'SECURITY', release: 'patch'},
        {tag: 'Breaking', release: 'minor'},
        {tag: 'Fix', release: 'patch'},
        {tag: 'Update', release: 'minor'},
        {tag: 'New', release: 'minor'},
        {component: 'perf', release: 'patch'},
        {component: 'deps', release: 'patch'},
        {type: 'FEAT', release: 'minor'},
        {type: 'FIX', release: 'patch'},
      ],
    }],
    '@semantic-release/release-notes-generator',
    ['@semantic-release/exec', {
      "prepareCmd": "bash version_update.sh ${nextRelease.version}",
      "publishCmd": "bash cargo_publish.sh",
    }],
    '@semantic-release/github',
  ],
};