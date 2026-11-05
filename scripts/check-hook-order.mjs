#!/usr/bin/env node
/**
 * Guards against React's "Rendered more hooks than during the previous render".
 *
 * A component that returns early (e.g. `if (!isOpen) return null`) *before* some
 * of its hooks will render a different number of hooks once the guard flips,
 * which crashes the app with an ErrorBoundary instead of a visible warning.
 * TypeScript cannot catch it and the project has no eslint react-hooks setup,
 * so this script walks the AST and reports every component whose top-level
 * `return` precedes a top-level hook call.
 *
 * Run: npm run lint:hooks
 */
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import ts from 'typescript';

const HOOK_RE = /^use[A-Z0-9]/;

function walk(dir, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'node_modules' || entry.name.startsWith('.')) continue;
    const full = join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else if (/\.tsx?$/.test(entry.name) && !entry.name.endsWith('.d.ts')) out.push(full);
  }
  return out;
}

const files = walk('src');

function calleeName(expr) {
  if (ts.isIdentifier(expr)) return expr.text;
  if (ts.isPropertyAccessExpression(expr)) return expr.name.text;
  return '';
}

/** Is this expression a direct `useX(...)` call? */
function isHookCall(expr) {
  return (
    ts.isCallExpression(expr) &&
    HOOK_RE.test(calleeName(expr.expression))
  );
}

/** Hook called directly by a top-level statement of the component body. */
function statementHookName(statement) {
  if (ts.isVariableStatement(statement)) {
    for (const decl of statement.declarationList.declarations) {
      if (decl.initializer && isHookCall(decl.initializer)) {
        return calleeName(decl.initializer.expression);
      }
    }
  }
  if (ts.isExpressionStatement(statement) && isHookCall(statement.expression)) {
    return calleeName(statement.expression.expression);
  }
  return null;
}

/** Does this statement bail out of the component before finishing? */
function statementBailsOut(statement) {
  if (ts.isReturnStatement(statement)) return true;
  // `if (cond) return null;` / `if (cond) { ... return ... }` at the top level
  if (ts.isIfStatement(statement)) {
    const branches = [statement.thenStatement, statement.elseStatement].filter(Boolean);
    return branches.some((branch) => {
      if (ts.isReturnStatement(branch)) return true;
      if (ts.isBlock(branch)) {
        return branch.statements.some((s) => ts.isReturnStatement(s) || ts.isThrowStatement(s));
      }
      return ts.isThrowStatement(branch);
    });
  }
  return false;
}

function componentFunction(node) {
  // const Foo: React.FC<...> = (...) => {}
  if (ts.isVariableDeclaration(node) && ts.isIdentifier(node.name)) {
    const name = node.name.text;
    if (/^[A-Z]/.test(name) && node.initializer) {
      if (ts.isArrowFunction(node.initializer) || ts.isFunctionExpression(node.initializer)) {
        return { name, fn: node.initializer };
      }
    }
  }
  // function Foo() {}
  if (ts.isFunctionDeclaration(node) && node.name && /^[A-Z]/.test(node.name.text)) {
    return { name: node.name.text, fn: node };
  }
  return null;
}

const problems = [];

for (const file of files) {
  const text = readFileSync(file, 'utf8');
  if (!text.includes('use')) continue;
  const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);

  const visit = (node) => {
    const component = componentFunction(node);
    if (component && component.fn.body && ts.isBlock(component.fn.body)) {
      const { name, fn } = component;
      const statements = fn.body.statements;
      const firstBailIdx = statements.findIndex((s) => statementBailsOut(s));
      const lastHookIdx = statements.findLastIndex((s) => statementHookName(s));
      // A guard that sits *before* any hook means the hook count changes when
      // the guard flips.
      if (firstBailIdx !== -1 && lastHookIdx > firstBailIdx) {
        const guard = statements[firstBailIdx];
        const hook = statementHookName(statements[lastHookIdx]);
        const guardLine = source.getLineAndCharacterOfPosition(guard.getStart(source)).line + 1;
        const hookLine = source.getLineAndCharacterOfPosition(statements[lastHookIdx].getStart(source)).line + 1;
        problems.push({
          file,
          line: guardLine,
          message: `component "${name}" returns early at line ${guardLine} but calls ${hook}() at line ${hookLine} — move the guard below every hook`,
        });
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(source);
}

if (problems.length) {
  for (const p of problems) console.error(`✖ ${p.file}:${p.line}  ${p.message}`);
  console.error(`\n${problems.length} hook-order problem(s) found.`);
  process.exit(1);
}
console.log(`✔ hook order OK (${files.length} files scanned)`);
