import { ESLint } from 'eslint';

const eslint = new ESLint();

const results = await eslint.lintFiles(['lib/**/*.{js,html}', '*.html']);

if (results.length > 0) {
  results.forEach(result => {
    if (result.errorCount > 0 || result.warningCount > 0) {
      console.log(result.messages);
    }
  });
} else {
  console.log('No lintable files found');
}