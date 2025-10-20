'use strict';
(function () {
    document.addEventListener('DOMContentLoaded', function () {
        introJs()
            .setOptions({
                dontShowAgain: true,
                dontShowAgainLabel: "Ne plus afficher",
                steps: [
                    {
                        title: 'Bienvenue',
                        intro: 'Découvrez comment utiliser Smartlinker.'
                    },
                    {
                        title: 'Votre SmartLink',
                        element: document.querySelector('.step1'),
                        intro: 'Le SmartLink est votre lien unique qui redirige vos sollicitations vers un formulaire de contact.'
                    },
                    {
                        title: 'Editez votre SmartLink',
                        element: document.querySelector('.step2'),
                        intro: 'SmartLink est un lien unique que vous pouvez éditer une seule fois. Pensez bien a enregistrer vos modifications si le lien vous convient.'
                    },
                    {
                        title: 'Activez SmartReply',
                        element: document.querySelector('.step3'),
                        intro: 'Une fois votre lien enregistré, activez SmartReply pour automatiser vos réponses emails.'
                    },
                    {
                        title: 'Consultez vos crédits',
                        element: document.querySelector('.step4'),
                        intro: 'Consultez vos crédits de réponses envoyées ici. Si vous avez atteint votre limite, vous pouvez passer premium.'
                    },
                    {
                        title: 'Votre réponse',
                        element: document.querySelector('.step5'),
                        intro: 'Vous pouvez a tout moment modifier votre réponse automatique ici.'
                    },
                    {
                        title: "Visualisez vos demandes récentes",
                        element: document.querySelector('.step6'),
                        intro: 'Vous pouvez visualiser vos propositions récentes reçues via votre SmartLink.'
                    }
                ]
            })
            .start();
    });
})();
