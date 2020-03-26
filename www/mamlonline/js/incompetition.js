$(() => {
    let answerarea = $("#answer-answers")
    for (probnum = 0; probnum < 5; probnum++) {
        let answerDiv = $("<div></div>")
        answerDiv.addClass("answer-row")

        let answerNumberSpan = $("<span></span>")
        answerNumberSpan.addClass("answer-number")
        answerNumberSpan.text((probnum + 1) + ": ")
        answerDiv.append(answerNumberSpan);

        let answerbox = $("<span></span>");
        answerbox.addClass("answer")
        answerDiv.append(answerbox);

        let mathField = MQ.MathField(answerbox.get(0), {
            spaceBehavesLikeTab: true,
            handlers: {
                edit: function() {
                    //todo: save
                }
            }
        });

        answerarea.append(answerDiv);

    }


    $("#answer-input-guide").dialog({
        autoOpen: false
    });

    $("#answer-input-guide-button").on("click", function() {
        $("#answer-input-guide").dialog("open");
    });

    $("#answer-input-guide").dialog({
        autoOpen: false
    });
})
