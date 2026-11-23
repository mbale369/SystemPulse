// Existing machine Playwright runtime; no browser or package download.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.SYSTEMPULSE_PLAYWRIGHT ?? '/home/kiunda/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const browser = await chromium.launch({executablePath:process.env.SYSTEMPULSE_CHROME ?? '/usr/bin/google-chrome',headless:true});
const errors=[];
const context=await browser.newContext({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
const page=await context.newPage();
page.on('pageerror',error=>errors.push(error.message));
page.on('console',message=>{if(message.type()==='error')errors.push(message.text());});
const external=[];page.on('request',request=>{if(!request.url().startsWith('http://127.0.0.1:1420/')&&!request.url().startsWith('data:'))external.push(request.url());});
try {
  await page.goto('http://127.0.0.1:1420/',{waitUntil:'networkidle'});
  await page.getByText('DÉMONSTRATION — données fictives',{exact:true}).waitFor();
  assert.equal(await page.locator('h1').count(),1);
  await page.waitForFunction(()=>document.querySelectorAll('.metric-value')[0]?.textContent!=='—');
  await page.waitForTimeout(3300);
  assert.equal(await page.locator('.uplot').count(),1);
  await mkdir('test-artifacts',{recursive:true});
  await page.screenshot({path:'test-artifacts/dashboard-demo.png',fullPage:true});
  for(const name of ['Processus','Diagnostics','Historique','Intégration IA','Paramètres']){
    await page.getByRole('button',{name,exact:true}).click();
    assert.equal(await page.locator('h1').count(),1);
    assert.equal(await page.locator('h1').textContent(),name);
  }
  await page.getByLabel('Langue',{exact:true}).selectOption('en');
  assert.equal(await page.locator('html').getAttribute('lang'),'en');
  await page.getByLabel('Appearance',{exact:true}).selectOption('light');
  assert.equal(await page.locator('html').getAttribute('data-theme'),'light');
  await page.getByRole('button',{name:'Processes',exact:true}).click();
  await page.getByRole('searchbox').fill('compilation');
  assert.equal(await page.locator('tbody tr').count(),1);
  await page.getByRole('searchbox').fill('<script>alert(1)</script>');
  assert.equal(await page.locator('tbody tr').count(),0);
  await page.getByRole('button',{name:'Overview',exact:true}).click();
  for(const width of [320,768,1024,1440]){
    await page.setViewportSize({width,height:900});await page.waitForTimeout(50);
    assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`Horizontal overflow at ${width}`);
  }
  await page.setViewportSize({width:320,height:900});await page.screenshot({path:'test-artifacts/mobile-demo.png',fullPage:true});
  await page.keyboard.press('Tab');
  assert.notEqual(await page.evaluate(()=>document.activeElement?.tagName),'BODY');
  assert.deepEqual(errors,[]);assert.deepEqual(external,[]);
  console.log(JSON.stringify({result:'PASS',checks:['six-screen navigation','live fixture update','uPlot graph','FR/EN','dark/light','process filter','malicious filter','320/768/1024/1440 responsive','keyboard focus','no console errors','no external requests'],screenshots:['test-artifacts/dashboard-demo.png','test-artifacts/mobile-demo.png']}));
} finally {await context.close();await browser.close();}
