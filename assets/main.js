document.addEventListener('DOMContentLoaded', function () {
    const form = document.getElementById('moveform');

    form.addEventListener('click', function (event) {
        let target = event.target;
        while (target !== form && target.type !== 'submit') {
            target = target.parentNode;
        }
        if (target.type === 'submit') {
            let hiddenInput = document.createElement('input');
            hiddenInput.type = 'hidden';
            hiddenInput.name = 'direction';
            hiddenInput.value = target.value;
            form.appendChild(hiddenInput);
        }
    });

    form.addEventListener('submit', function (event) {
        event.preventDefault();
        const repaint = async () => {
            for (let i = 0; i < 2; i++) {
                await new Promise(resolve => requestAnimationFrame(resolve));
            }
        };

        (async () => {
            document.getElementById('loading').style.display = 'inline'; // ローディング画像を表示
            await repaint(); // 画面を再描画して待つ
            form.removeEventListener('submit', arguments.callee);
            form.submit();
        })();

    });
});
