#include <stdio.h>
#include <stdbool.h>
#include "wrapper.h"

int main() 
{
	printf("Initilizing:\n");
	void* story = Step(NULL); 

	printf("Status:\n\t%s\n\tContinue?: %s\n\tChoice count: %i\n", 
			story == NULL ? "Ended" : "Unfinished", 
			CanContinue(story) ? "True":"False"	,
			ChoiceCount(story)
	);

	printf("Stepping..\n");
	story = Step(story);
	printf("Status:\n\t%s\n\tContinue?: %s\n\tChoice count: %i\n", 
			story == NULL ? "Ended" : "Unfinished", 
			CanContinue(story) ? "True":"False"	,
			ChoiceCount(story)
	);

	unsigned int choice = 1;
	printf("Choosing choice %i...\n", choice);
	ChooseChoiceIndex(story, choice);
	printf("Status:\n\t%s\n\tContinue?: %s\n\tChoice count: %i\n", 
			story == NULL ? "Ended" : "Unfinished", 
			CanContinue(story) ? "True":"False"	,
			ChoiceCount(story)
	);

	printf("Stepping..\n");
	story = Step(story);
	
	printf("Status:\n\t%s\n\tContinue?: %s\n\tChoice count: %i\n", 
			story == NULL ? "Ended" : "Unfinished", 
			CanContinue(story) ? "True":"False"	,
			ChoiceCount(story)
	);

	return 0;
}
