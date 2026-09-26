import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
import { startServer } from './serve.mjs';
const server = await startServer(0, '/playground/');
const browser = await chromium.launch({headless: true});
const url = 'http://127.0.0.1:' + server.address().port + '/playground/?debug=1';
async function loaded(page) {
  await page.goto(url);
  await page.waitForFunction(() => document.querySelector('#source').value.includes('HELLO'));
}
try {
  const page = await browser.newPage();
  await loaded(page);
  await page.locator('#run').click();
  await page.waitForFunction(() => document.querySelector('#history-count').textContent === '(1)');
  assert.match(await page.locator('#status').innerText(), /ast: ok/);
  await page.locator('#source').press('Control+Enter');
  await page.waitForFunction(() => document.querySelector('#history-count').textContent === '(2)');
  console.log('PASS button and keyboard execute on a subpath');
  await page.close();

  for (const asset of ['**/*.wasm', '**/pkg/cobol_transformer_wasm.js']) {
    const page = await browser.newPage();
    await page.route(asset, route => route.abort());
    await page.goto(url);
    await page.waitForFunction(() => document.querySelector('#wasm-status').textContent.includes('failed'));
    await page.locator('#run').click();
    assert.equal(await page.locator('#run-label').innerText(), 'Execute');
    assert.match(await page.locator('#status').innerText(), /Cannot execute/);
    console.log('PASS failed asset does not queue forever: ' + asset);
    await page.close();
  }

  const missing = await browser.newPage();
  await loaded(missing);
  const original = await missing.locator('#source').inputValue();
  await missing.route('**/samples/vector-add.cbl', route => route.fulfill({status:404,body:'not found'}));
  await missing.locator('#sample-select').selectOption('vector-add');
  await missing.waitForFunction(() => document.querySelector('#sample-note').textContent.includes('HTTP 404'));
  assert.equal(await missing.locator('#source').inputValue(), original);
  console.log('PASS sample HTTP failure preserves source');
  await missing.close();

  const delayed = await browser.newPage();
  let release, started;
  const startedPromise = new Promise(resolve => {started = resolve;});
  const gate = new Promise(resolve => {release = resolve;});
  await delayed.route('**/*.wasm', async route => {started(); await gate; await route.continue();});
  await delayed.goto(url, {waitUntil:'domcontentloaded'});
  await startedPromise;
  await delayed.locator('#source').fill(original.replaceAll('HELLO','USERCODE'));
  await delayed.locator('#run').click();
  assert.match(await delayed.locator('#run-label').innerText(), /^Queued/);
  release();
  await delayed.waitForFunction(() => document.querySelector('#history-count').textContent === '(1)');
  assert.match(await delayed.locator('#source').inputValue(), /USERCODE/);
  assert.match(await delayed.locator('#status').innerText(), /program USERCODE/);
  console.log('PASS queued startup run preserves typed source');
  await delayed.close();

  const racing = await browser.newPage();
  await loaded(racing);
  let releaseSample, sampleStarted;
  const sampleStartedPromise = new Promise(resolve => {sampleStarted = resolve;});
  const sampleGate = new Promise(resolve => {releaseSample = resolve;});
  await racing.route('**/samples/ledger-post.cbl', async route => {sampleStarted(); await sampleGate; await route.continue();});
  await racing.locator('#sample-select').selectOption('ledger-post');
  await sampleStartedPromise;
  await racing.locator('#sample-select').selectOption('hello');
  await racing.waitForFunction(() => document.querySelector('#debug-log').textContent.includes('run-end'));
  releaseSample();
  await racing.waitForFunction(() => document.querySelector('#debug-log').textContent.includes('sample-discarded'));
  assert.match(await racing.locator('#source').inputValue(), /PROGRAM-ID. HELLO/);
  console.log('PASS stale sample download cannot replace newer selection');
  await racing.close();
} finally {
  await browser.close();
  await new Promise(resolve => server.close(resolve));
}
