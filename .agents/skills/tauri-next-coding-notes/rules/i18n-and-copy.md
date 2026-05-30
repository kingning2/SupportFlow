# i18n and Copy

## Rule

- 可见文案必须走 `useTranslation`。
- key 按 namespace 管理，不要平铺。
- 新增 key 同步更新 `cn.json` 与 `en.json`。

## Incorrect

```tsx
<h1>立即購買</h1>
```

## Correct

```tsx
const { t } = useTranslation('title_bar')
<h1>{t('menu_about')}</h1>
```
